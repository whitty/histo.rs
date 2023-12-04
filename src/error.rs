#[derive(Debug)]
pub enum Error {
    VarError(std::env::VarError),
    ParseIntError(std::num::ParseIntError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Error::ParseIntError(e) =>
                write!(f, "Failed to parse {}", e),
            Error::VarError(e) =>
                write!(f, "Environment variable error {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::ParseIntError(ref e) => Some(e),
            Error::VarError(ref e) => Some(e),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Error {
        Error::VarError(err)
    }
}
