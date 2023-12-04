pub mod data;
pub mod graph;
mod error;

type Result<T> = std::result::Result<T, error::Error>;
