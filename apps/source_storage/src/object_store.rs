use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::Utc;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::{Error, FileFormat, Result, StorageIdentity};

/// Local raw-file storage. Keys are always UUIDs and supported file extensions.
#[derive(Debug, Clone)]
pub struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    /// Creates the root/raw directory and resolves the root to an absolute path.
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(root.as_ref().join("raw"))?;
        let root = fs::canonicalize(root)?;
        if root.to_str().is_none() {
            return Err(Error::NonUtf8Path);
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, id: Uuid, format: FileFormat) -> PathBuf {
        self.root.join("raw").join(format!("{id}.{format}"))
    }

    /// Publishes a fully written file without overwriting an existing object.
    pub fn put(&self, id: Uuid, format: FileFormat, bytes: &[u8]) -> Result<StorageIdentity> {
        let path = self.path(id, format);
        let storage_uri = path.to_str().ok_or(Error::NonUtf8Path)?.to_owned();
        let mut temporary = NamedTempFile::new_in(self.root.join("raw"))?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist_noclobber(&path).map_err(|error| {
            if error.error.kind() == std::io::ErrorKind::AlreadyExists {
                Error::ObjectAlreadyExists(id)
            } else {
                Error::Io(error.error)
            }
        })?;
        Ok(StorageIdentity {
            id,
            storage_uri,
            file_format: format,
            content_hash: Self::content_hash(bytes),
            lastfetched_at: Utc::now(),
        })
    }

    pub fn read(&self, id: Uuid, format: FileFormat) -> Result<Vec<u8>> {
        Ok(fs::read(self.path(id, format))?)
    }

    pub fn content_hash(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    pub(crate) fn remove(&self, id: Uuid, format: FileFormat) -> std::io::Result<()> {
        fs::remove_file(self.path(id, format))
    }
}
