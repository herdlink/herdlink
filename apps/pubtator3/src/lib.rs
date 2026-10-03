//! Typed asynchronous access to [PubTator3](https://www.ncbi.nlm.nih.gov/research/pubtator3/api).
//!
//! ```no_run
//! use pubtator3::{Client, Pmid, TextScope};
//!
//! # async fn example() -> pubtator3::Result<()> {
//! let client = Client::new()?;
//! let documents = client.annotations(&[Pmid::new(19894120)?], TextScope::Abstract).await?;
//! for document in documents {
//!     for annotation in document.annotations() {
//!         println!("{}: {:?}", annotation.text, annotation.infons.accession);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! MeSH entry terms can also be listed directly by descriptor ID:
//! ```no_run
//! use pubtator3::{Client, MeshDescriptorId};
//! # async fn example() -> pubtator3::Result<()> {
//! let id: MeshDescriptorId = "D000690".parse()?;
//! let terms = Client::new()?.mesh_synonyms(&id).await?;
//! for term in terms.synonyms() {
//!     println!("{}: {}", term.id, term.label);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod error;
mod ids;
mod models;
mod query;

pub use client::{Client, ClientBuilder, DEFAULT_BASE_URL, DEFAULT_MESH_BASE_URL, MAX_EXPORT_IDS};
pub use error::{Error, Result};
pub use ids::*;
pub use models::*;
pub use query::*;
