//! Typed SQLite records and local raw-file storage for Herdlink.
//!
//! See [`Client::insert_with_file`] to store bytes and metadata together, or
//! [`ObjectStore::put`] followed by [`Client::insert`] for separate operations.

mod client;
mod error;
mod models;
mod object_store;

pub use client::Client;
pub use error::{Error, Result};
pub use models::*;
pub use object_store::ObjectStore;
