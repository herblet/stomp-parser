#![warn(clippy::all)]
mod model;
mod parser;

pub mod error;

pub use model::client;
pub use model::headers;
pub use model::server;
