//! Map PubTator diseases to HPO profiles and rank diseases by phenotype similarity.
//! Mapping and scoring are separate: exact identifiers are preferred; lexical matches
//! retain their provenance. Similarity is not a PubTator relation or a diagnostic probability.
//!
//! ```no_run
//! use pubtator3_hpo::{Client, Dataset, SimilarityOptions};
//! # async fn example() -> pubtator3_hpo::Result<()> {
//! let dataset = Dataset::from_dir("./phenotype-data")?;
//! let client = Client::new(pubtator3::Client::new()?, dataset);
//! let report = client.similar_diseases("Huntington disease", None,
//!     &SimilarityOptions::default()).await?;
//! println!("Mapping: {:?}", report.mapping);
//! for disease in report.matches {
//!     println!("{}: {:.3} — {}", disease.disease_id, disease.score, disease.name);
//! }
//! # Ok(())
//! # }
//! ```

mod annotations;
mod client;
mod dataset;
mod download;
mod error;
mod ids;
mod mapper;
mod models;
mod mondo;
mod ontology;

pub use client::{Client, MappedDisease, PubTatorDiseaseMatch};
pub use dataset::Dataset;
pub use download::{DataFile, DataManifest, download_data};
pub use error::{Error, Result};
pub use ids::{DiseaseId, HpoId, MeshId, MondoId};
pub use mapper::DiseaseMapper;
pub use models::*;
