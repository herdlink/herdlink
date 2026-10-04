//! Neo4j-backed read-through caches and extensible biomedical graph projections.
mod clients;
mod model;
mod projection;
mod store;

pub use clients::{CachePolicy, CachedHpo, CachedPubTator};
pub use model::{Edge, GraphBatch, Node, Properties};
pub use projection::{
    documents, entities, mapped_disease, mesh_terms, ontology_terms, profile, relations, search,
};
pub use store::Store;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Neo4j(#[from] neo4rs::Error),
    #[error(transparent)]
    Neo4jDecode(#[from] neo4rs::DeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    PubTator(#[from] pubtator3::Error),
    #[error(transparent)]
    Hpo(#[from] pubtator3_hpo::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Task(#[from] tokio::task::JoinError),
    #[error("invalid graph data: {0}")]
    Invalid(String),
}
pub type Result<T> = std::result::Result<T, Error>;
