pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid {kind}: {value:?}")]
    InvalidId { kind: &'static str, value: String },
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error(
        "synonym lookup requires a MeSH descriptor; entity {entity} uses {database:?} ID {database_id:?}"
    )]
    UnsupportedSynonyms {
        entity: crate::EntityId,
        database: Option<String>,
        database_id: Option<String>,
    },
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("API returned HTTP {status}: {body}")]
    Http {
        status: reqwest::StatusCode,
        body: String,
        /// Unmodified Retry-After header (seconds or an HTTP date), if supplied.
        retry_after: Option<String>,
    },
    #[error("invalid API JSON: {0}")]
    Json(#[from] serde_json::Error),
}
