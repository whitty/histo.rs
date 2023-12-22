pub mod data;
pub mod graph;
pub mod error;

pub type Error = error::Error;
type Result<T> = std::result::Result<T, error::Error>;
