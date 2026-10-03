use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid {field}: {value}")]
    InvalidValue { field: &'static str, value: String },
    #[error("record {0} does not exist")]
    NotFound(Uuid),
    #[error("object already exists: {0}")]
    ObjectAlreadyExists(Uuid),
    #[error("object checksum does not match record {0}")]
    ChecksumMismatch(Uuid),
    #[error("object path is not valid UTF-8")]
    NonUtf8Path,
    #[error("insert failed ({insert}); object cleanup also failed ({cleanup})")]
    CleanupFailed {
        insert: Box<Error>,
        cleanup: std::io::Error,
    },
}
