use crate::Response;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read auth.json: {0}")]
    AuthFileIo(#[source] std::io::Error),
    #[error("invalid client configuration: {0}")]
    Configuration(String),
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
    #[error("OpenAI HTTP {status} (request ID: {request_id:?}): {body}")]
    Http {
        status: u16,
        request_id: Option<String>,
        body: String,
    },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid event stream: {0}")]
    Stream(String),
    #[error("OpenAI stream error ({code:?}): {message}")]
    Api {
        code: Option<String>,
        message: String,
    },
    #[error("response did not complete successfully: {0:?}")]
    UnsuccessfulResponse(Box<Response>),
    #[error("stream ended before a terminal response event")]
    UnexpectedEof,
}
