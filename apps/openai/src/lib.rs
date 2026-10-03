//! Small async OpenAI Responses API client with SSE streaming and function tools.
//!
//! Tools are declarations only: the caller executes functions and returns their
//! results using [`InputItem::tool_output`] and [`ResponseRequest::continue_from`].

mod auth;
mod client;
mod error;
mod models;

pub use client::{Client, ClientBuilder, ResponseStream};
pub use error::{Error, Result};
pub use models::*;
