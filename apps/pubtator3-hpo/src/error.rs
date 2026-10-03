pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid identifier: {0}")]
    InvalidId(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("invalid dataset: {0}")]
    Data(String),
    #[error("no disease match for {0:?}")]
    NoDisease(String),
    #[error("ambiguous disease {query:?}; select an entity ID from {candidates:?}")]
    AmbiguousDisease {
        query: String,
        candidates: Vec<pubtator3::EntityId>,
    },
    #[error("no HPO mapping: {0}")]
    Unmapped(String),
    #[error("ambiguous HPO mapping; candidates: {0:?}")]
    AmbiguousMapping(Vec<crate::DiseaseMapping>),
    #[error("disease has no positive phenotypic-abnormality annotations: {0}")]
    NoPhenotypes(crate::DiseaseId),
    #[error(transparent)]
    PubTator(#[from] pubtator3::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("HPO ontology error: {0:?}")]
    Hpo(hpo::HpoError),
    #[error("dataset worker failed: {0}")]
    Worker(#[from] tokio::task::JoinError),
}

impl From<hpo::HpoError> for Error {
    fn from(error: hpo::HpoError) -> Self {
        Self::Hpo(error)
    }
}
