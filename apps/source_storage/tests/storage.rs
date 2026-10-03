use chrono::{DateTime, NaiveDate, Utc};
use serde_json::json;
use source_storage::{
    Author, Category, Classification, Client, Content, Error, FileFormat, NewRecord, ObjectStore,
    RecordStatus, Source,
};
use tempfile::tempdir;
use uuid::Uuid;

fn input() -> NewRecord {
    NewRecord {
        file_format: FileFormat::Pdf,
        classification: Classification {
            category: Category::Research,
            subtype: "case_report".into(),
            is_peer_reviewed: true,
        },
        source: Source {
            source_name: "pubmed".into(),
            external_id: "12345678".into(),
            doi: Some("10.1234/example".into()),
            source_url: "https://pubmed.ncbi.nlm.nih.gov/12345678/".into(),
        },
        content: Content {
            title: "A rare disease case report — α".into(),
            summary: Some("Abstract with 'quotes' and Unicode.".into()),
            language: "en".into(),
            authors: Some(vec![Author {
                name: "A. Researcher".into(),
                orcid: Some("0000-0002-1825-0097".into()),
                affiliations: vec!["University".into()],
            }]),
            published_at: Some(NaiveDate::from_ymd_opt(2026, 9, 30).unwrap()),
            source_updated_at: Some(
                "2026-10-03T12:34:56.123456789Z"
                    .parse::<DateTime<Utc>>()
                    .unwrap(),
            ),
            record_status: RecordStatus::Active,
            metadata: json!({"journal": "Example", "mesh_terms": ["Rare Diseases"]}),
        },
    }
}

#[test]
fn round_trip_file_metadata_and_flat_json() {
    let directory = tempdir().unwrap();
    let client = Client::in_memory(directory.path()).unwrap();
    let bytes = b"%PDF-1.7\nexample\0\xff";
    let record = client.insert_with_file(input(), bytes).unwrap();
    assert_eq!(client.get(record.storage.id).unwrap(), Some(record.clone()));
    assert_eq!(client.read_file(record.storage.id).unwrap(), bytes);
    assert_eq!(
        client
            .object_store()
            .read(record.storage.id, FileFormat::Pdf)
            .unwrap(),
        bytes
    );
    assert_eq!(
        record.storage.storage_uri,
        directory
            .path()
            .join("raw")
            .join(format!("{}.pdf", record.storage.id))
            .to_str()
            .unwrap()
    );
    let value = serde_json::to_value(&record).unwrap();
    assert_eq!(value["category"], "research");
    assert_eq!(value["id"], record.storage.id.to_string());
    assert_eq!(
        serde_json::from_value::<source_storage::Record>(value).unwrap(),
        record
    );
    assert_eq!(
        ObjectStore::content_hash(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn all_categories_formats_and_statuses_round_trip() {
    let directory = tempdir().unwrap();
    let client = Client::in_memory(directory.path()).unwrap();
    let categories = [
        Category::Research,
        Category::Preprint,
        Category::ClinicalStudy,
        Category::Funding,
        Category::Reference,
        Category::ModelResource,
        Category::Organization,
        Category::Experience,
        Category::News,
        Category::Guideline,
    ];
    let formats = [
        FileFormat::Pdf,
        FileFormat::Xml,
        FileFormat::Json,
        FileFormat::Html,
    ];
    let statuses = [
        RecordStatus::Active,
        RecordStatus::Retracted,
        RecordStatus::Withdrawn,
        RecordStatus::Terminated,
    ];
    for (index, category) in categories.into_iter().enumerate() {
        let mut input = input();
        input.classification.category = category;
        input.classification.is_peer_reviewed = false;
        input.file_format = formats[index % formats.len()];
        input.content.record_status = statuses[index % statuses.len()];
        input.source.doi = None;
        input.content.summary = None;
        input.content.authors = if index % 2 == 0 { None } else { Some(vec![]) };
        input.content.published_at = None;
        input.content.source_updated_at = None;
        input.content.metadata = json!({});
        let record = client.insert_with_file(input, b"example").unwrap();
        assert_eq!(client.get(record.storage.id).unwrap(), Some(record.clone()));
        assert_eq!(client.find_by_category(category).unwrap(), vec![record]);
    }
    assert_eq!(client.list(100, 0).unwrap().len(), categories.len());
}

#[test]
fn duplicate_detection_and_pagination_keep_distinct_sources() {
    let directory = tempdir().unwrap();
    let client = Client::in_memory(directory.path()).unwrap();
    let first = client.insert_with_file(input(), b"same bytes").unwrap();
    let mut other = input();
    other.source.source_name = "orphanet".into();
    let second = client.insert_with_file(other, b"same bytes").unwrap();
    assert_eq!(
        client.find_by_source("pubmed", "12345678").unwrap(),
        vec![first.clone()]
    );
    assert_eq!(client.find_by_doi("10.1234/example").unwrap().len(), 2);
    assert_eq!(
        client
            .find_by_content_hash(&first.storage.content_hash)
            .unwrap()
            .len(),
        2
    );
    let all = client.list(100, 0).unwrap();
    assert_eq!(all.len(), 2);
    assert!(all[0].storage.id.to_string() < all[1].storage.id.to_string());
    assert_eq!(client.list(1, 1).unwrap(), all[1..]);
    assert!(client.list(0, 0).unwrap().is_empty());
    assert!(client.list(10, 2).unwrap().is_empty());
    assert!(client.find_by_doi("missing").unwrap().is_empty());
    assert!(client.insert(&first).is_err());
    assert_eq!(client.get(first.storage.id).unwrap(), Some(first));
    assert_eq!(client.get(second.storage.id).unwrap(), Some(second));
}

#[test]
fn separate_object_and_record_inserts_and_updates_persist_after_reopening() {
    let directory = tempdir().unwrap();
    let database = directory.path().join("records.sqlite");
    let root = directory.path().join("objects");
    let client = Client::open(&database, &root).unwrap();
    let input = input();
    let storage = client
        .object_store()
        .put(Uuid::new_v4(), input.file_format, b"raw")
        .unwrap();
    let mut record = source_storage::Record {
        storage,
        classification: input.classification,
        source: input.source,
        content: input.content,
    };
    client.insert(&record).unwrap();
    record.content.title = "Updated title".into();
    record.content.record_status = RecordStatus::Retracted;
    client.update(&record).unwrap();
    drop(client);
    let client = Client::open(&database, &root).unwrap();
    assert_eq!(client.get(record.storage.id).unwrap(), Some(record.clone()));
    assert_eq!(client.read_file(record.storage.id).unwrap(), b"raw");
    let missing = Uuid::new_v4();
    assert!(client.get(missing).unwrap().is_none());
    assert!(matches!(client.read_file(missing), Err(Error::NotFound(id)) if id == missing));
    record.storage.id = missing;
    assert!(matches!(client.update(&record), Err(Error::NotFound(id)) if id == missing));
}

#[test]
fn failed_database_insert_cleans_up_object() {
    let directory = tempdir().unwrap();
    let client = Client::in_memory(directory.path()).unwrap();
    let mut input = input();
    input.content.metadata = json!(["not an object"]);
    assert!(matches!(
        client.insert_with_file(input, b"raw"),
        Err(Error::Sqlite(_))
    ));
    assert!(client.list(10, 0).unwrap().is_empty());
    assert_eq!(
        std::fs::read_dir(directory.path().join("raw"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn object_store_never_overwrites_and_file_reads_detect_corruption() {
    let directory = tempdir().unwrap();
    let client = Client::in_memory(directory.path()).unwrap();
    let record = client.insert_with_file(input(), b"original").unwrap();
    let id = record.storage.id;
    assert!(
        matches!(client.object_store().put(id, FileFormat::Pdf, b"replacement"),
        Err(Error::ObjectAlreadyExists(existing)) if existing == id)
    );
    assert_eq!(client.read_file(id).unwrap(), b"original");
    std::fs::write(&record.storage.storage_uri, b"corrupted").unwrap();
    assert!(matches!(client.read_file(id), Err(Error::ChecksumMismatch(bad)) if bad == id));
    std::fs::remove_file(&record.storage.storage_uri).unwrap();
    assert!(matches!(client.read_file(id), Err(Error::Io(_))));
}

#[test]
fn raw_sql_constraints_and_malformed_rows_are_rejected() {
    let directory = tempdir().unwrap();
    let database = directory.path().join("records.sqlite");
    let client = Client::open(&database, directory.path().join("objects")).unwrap();
    let record = client.insert_with_file(input(), b"raw").unwrap();
    let connection = rusqlite::Connection::open(database).unwrap();
    for assignment in [
        "category = 'invalid'",
        "file_format = 'exe'",
        "is_peer_reviewed = 2",
        "content_hash = 'invalid'",
        "record_status = 'invalid'",
        "authors = '{}'",
        "authors = 'invalid'",
        "metadata = 'null'",
        "metadata = 'invalid'",
    ] {
        assert!(
            connection
                .execute(&format!("UPDATE records SET {assignment}"), [])
                .is_err(),
            "accepted {assignment}"
        );
    }
    connection
        .execute("UPDATE records SET published_at = 'invalid'", [])
        .unwrap();
    assert!(matches!(
        client.get(record.storage.id),
        Err(Error::Sqlite(rusqlite::Error::FromSqlConversionFailure(
            _,
            _,
            _
        )))
    ));
}

#[test]
fn file_reads_reject_paths_outside_the_object_store() {
    let directory = tempdir().unwrap();
    let client = Client::in_memory(directory.path()).unwrap();
    let mut record = client.insert_with_file(input(), b"raw").unwrap();
    record.storage.storage_uri = "/etc/passwd".into();
    client.update(&record).unwrap();
    assert!(matches!(
        client.read_file(record.storage.id),
        Err(Error::InvalidValue {
            field: "storage_uri",
            ..
        })
    ));
}
