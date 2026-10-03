use std::{path::Path, str::FromStr, time::Duration};

use rusqlite::{Connection, OptionalExtension, Row, named_params, types::Type};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{
    Category, Classification, Content, Error, NewRecord, ObjectStore, Record, Result, Source,
    StorageIdentity,
};

const COLUMNS: &str = "id, storage_uri, file_format, content_hash, lastfetched_at,
    category, subtype, is_peer_reviewed, source_name, external_id, doi, source_url,
    title, summary, language, authors, published_at, source_updated_at, record_status, metadata";

/// Synchronous SQLite client owning a connection and a local object store.
/// Opening a client initializes the schema if it is not already present.
pub struct Client {
    connection: Connection,
    objects: ObjectStore,
}

impl Client {
    /// The database's parent directory must already exist.
    pub fn open(database: impl AsRef<Path>, object_root: impl AsRef<Path>) -> Result<Self> {
        Self::initialize(Connection::open(database)?, ObjectStore::new(object_root)?)
    }

    /// An ephemeral database with real filesystem object storage.
    pub fn in_memory(object_root: impl AsRef<Path>) -> Result<Self> {
        Self::initialize(
            Connection::open_in_memory()?,
            ObjectStore::new(object_root)?,
        )
    }

    fn initialize(connection: Connection, objects: ObjectStore) -> Result<Self> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch(include_str!("schema.sql"))?;
        Ok(Self {
            connection,
            objects,
        })
    }

    pub fn object_store(&self) -> &ObjectStore {
        &self.objects
    }

    /// Inserts metadata only. Call `insert_with_file` to also store the raw bytes.
    /// Duplicate IDs return a SQLite constraint error and never replace records.
    pub fn insert(&self, record: &Record) -> Result<()> {
        self.write(record, false)?;
        Ok(())
    }

    /// Stores bytes under a generated UUID, computes their hash, and inserts metadata.
    /// If the database insert fails, the newly created file is removed.
    /// SQLite and the filesystem cannot share a transaction: an abrupt process exit
    /// between the file write and the insert can leave an orphaned file.
    pub fn insert_with_file(&self, input: NewRecord, bytes: &[u8]) -> Result<Record> {
        let storage = self.objects.put(Uuid::new_v4(), input.file_format, bytes)?;
        let record = Record {
            storage,
            classification: input.classification,
            source: input.source,
            content: input.content,
        };
        if let Err(insert) = self.insert(&record) {
            if let Err(cleanup) = self
                .objects
                .remove(record.storage.id, record.storage.file_format)
            {
                return Err(Error::CleanupFailed {
                    insert: Box::new(insert),
                    cleanup,
                });
            }
            return Err(insert);
        }
        Ok(record)
    }

    /// Updates an existing row's metadata, without modifying its raw file.
    pub fn update(&self, record: &Record) -> Result<()> {
        if self.write(record, true)? == 0 {
            return Err(Error::NotFound(record.storage.id));
        }
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<Option<Record>> {
        Ok(self
            .connection
            .query_row(
                &format!("SELECT {COLUMNS} FROM records WHERE id = ?1"),
                [id.to_string()],
                record_from_row,
            )
            .optional()?)
    }

    /// Lists rows in UUID order using bounded pagination.
    pub fn list(&self, limit: u32, offset: u32) -> Result<Vec<Record>> {
        self.query(
            "ORDER BY id LIMIT ?1 OFFSET ?2",
            rusqlite::params![limit, offset],
        )
    }

    pub fn find_by_source(&self, source_name: &str, external_id: &str) -> Result<Vec<Record>> {
        self.query(
            "WHERE source_name = ?1 AND external_id = ?2 ORDER BY id",
            [source_name, external_id],
        )
    }

    /// Exact lookup of a DOI as stored, allowing records from multiple sources.
    pub fn find_by_doi(&self, doi: &str) -> Result<Vec<Record>> {
        self.query("WHERE doi = ?1 ORDER BY id", [doi])
    }

    pub fn find_by_content_hash(&self, hash: &str) -> Result<Vec<Record>> {
        self.query("WHERE content_hash = ?1 ORDER BY id", [hash])
    }

    pub fn find_by_category(&self, category: Category) -> Result<Vec<Record>> {
        self.query("WHERE category = ?1 ORDER BY id", [category.as_str()])
    }

    /// Loads a locally managed file and checks its SHA-256 digest against the row.
    /// Arbitrary paths in `storage_uri` are never followed.
    pub fn read_file(&self, id: Uuid) -> Result<Vec<u8>> {
        let record = self.get(id)?.ok_or(Error::NotFound(id))?;
        let path = self.objects.path(id, record.storage.file_format);
        if path.to_str() != Some(record.storage.storage_uri.as_str()) {
            return Err(Error::InvalidValue {
                field: "storage_uri",
                value: record.storage.storage_uri,
            });
        }
        let bytes = self.objects.read(id, record.storage.file_format)?;
        if ObjectStore::content_hash(&bytes) != record.storage.content_hash {
            return Err(Error::ChecksumMismatch(id));
        }
        Ok(bytes)
    }

    fn query(&self, suffix: &str, params: impl rusqlite::Params) -> Result<Vec<Record>> {
        let mut statement = self
            .connection
            .prepare(&format!("SELECT {COLUMNS} FROM records {suffix}"))?;
        Ok(statement
            .query_map(params, record_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn write(&self, record: &Record, update: bool) -> Result<usize> {
        let storage = &record.storage;
        let classification = &record.classification;
        let source = &record.source;
        let content = &record.content;
        let authors = content
            .authors
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let metadata = serde_json::to_string(&content.metadata)?;
        let sql = if update {
            "UPDATE records SET
                storage_uri = :storage_uri, file_format = :file_format,
                content_hash = :content_hash, lastfetched_at = :lastfetched_at,
                category = :category, subtype = :subtype, is_peer_reviewed = :is_peer_reviewed,
                source_name = :source_name, external_id = :external_id, doi = :doi,
                source_url = :source_url, title = :title, summary = :summary,
                language = :language, authors = :authors, published_at = :published_at,
                source_updated_at = :source_updated_at, record_status = :record_status,
                metadata = :metadata WHERE id = :id"
                .to_owned()
        } else {
            format!(
                "INSERT INTO records ({COLUMNS}) VALUES (
                :id, :storage_uri, :file_format, :content_hash, :lastfetched_at,
                :category, :subtype, :is_peer_reviewed, :source_name, :external_id,
                :doi, :source_url, :title, :summary, :language, :authors,
                :published_at, :source_updated_at, :record_status, :metadata)"
            )
        };
        Ok(self.connection.execute(
            &sql,
            named_params! {
                ":id": storage.id.to_string(),
                ":storage_uri": storage.storage_uri,
                ":file_format": storage.file_format.as_str(),
                ":content_hash": storage.content_hash,
                ":lastfetched_at": storage.lastfetched_at.to_rfc3339(),
                ":category": classification.category.as_str(),
                ":subtype": classification.subtype,
                ":is_peer_reviewed": classification.is_peer_reviewed,
                ":source_name": source.source_name,
                ":external_id": source.external_id,
                ":doi": source.doi,
                ":source_url": source.source_url,
                ":title": content.title,
                ":summary": content.summary,
                ":language": content.language,
                ":authors": authors,
                ":published_at": content.published_at.map(|date| date.to_string()),
                ":source_updated_at": content.source_updated_at.map(|date| date.to_rfc3339()),
                ":record_status": content.record_status.as_str(),
                ":metadata": metadata,
            },
        )?)
    }
}

fn parse<T>(row: &Row<'_>, column: &str) -> rusqlite::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value: String = row.get(column)?;
    value
        .parse()
        .map_err(|error| conversion_error(row, column, error))
}

fn parse_optional<T>(row: &Row<'_>, column: &str) -> rusqlite::Result<Option<T>>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value: Option<String> = row.get(column)?;
    value
        .map(|value| {
            value
                .parse()
                .map_err(|error| conversion_error(row, column, error))
        })
        .transpose()
}

fn json<T: DeserializeOwned>(row: &Row<'_>, column: &str) -> rusqlite::Result<T> {
    let value: String = row.get(column)?;
    serde_json::from_str(&value).map_err(|error| conversion_error(row, column, error))
}

fn conversion_error(
    row: &Row<'_>,
    column: &str,
    error: impl std::error::Error + Send + Sync + 'static,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        row.as_ref().column_index(column).unwrap_or(0),
        Type::Text,
        Box::new(error),
    )
}

fn record_from_row(row: &Row<'_>) -> rusqlite::Result<Record> {
    let authors: Option<String> = row.get("authors")?;
    Ok(Record {
        storage: StorageIdentity {
            id: parse(row, "id")?,
            storage_uri: row.get("storage_uri")?,
            file_format: parse(row, "file_format")?,
            content_hash: row.get("content_hash")?,
            lastfetched_at: parse(row, "lastfetched_at")?,
        },
        classification: Classification {
            category: parse(row, "category")?,
            subtype: row.get("subtype")?,
            is_peer_reviewed: row.get("is_peer_reviewed")?,
        },
        source: Source {
            source_name: row.get("source_name")?,
            external_id: row.get("external_id")?,
            doi: row.get("doi")?,
            source_url: row.get("source_url")?,
        },
        content: Content {
            title: row.get("title")?,
            summary: row.get("summary")?,
            language: row.get("language")?,
            authors: authors
                .map(|value| {
                    serde_json::from_str(&value)
                        .map_err(|error| conversion_error(row, "authors", error))
                })
                .transpose()?,
            published_at: parse_optional(row, "published_at")?,
            source_updated_at: parse_optional(row, "source_updated_at")?,
            record_status: parse(row, "record_status")?,
            metadata: json(row, "metadata")?,
        },
    })
}
