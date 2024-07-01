// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

#[derive(Debug)]
pub enum Error {
    NoData,
    DataTagsTooLongToFitTerminal(usize),
    VarError(std::env::VarError),
    IOError(std::io::Error),
    FormatError(std::fmt::Error),
    ParseIntError(std::num::ParseIntError),
    ScopedMatchCountError(String, String),
}

impl Error {
    pub fn no_data() -> Self {
        Error::NoData
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Error::NoData =>
                write!(f, "No data found, check your inputs and selectors"),
            Error::DataTagsTooLongToFitTerminal(u) =>
                write!(f, "Unable to fit graph in terminal width: {}", u),
            Error::ParseIntError(e) =>
                write!(f, "Failed to parse {}", e),
            Error::VarError(e) =>
                write!(f, "Environment variable error {}", e),
            Error::IOError(e) =>
                write!(f, "I/O error {}", e),
            Error::FormatError(e) =>
                write!(f, "Format error {}", e),
            Error::ScopedMatchCountError(i, o) =>
                write!(f, "Scoped regexes don't have matching captures '{}' '{}'", i, o),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::NoData | Error::DataTagsTooLongToFitTerminal(_) |
            Error::ScopedMatchCountError(_, _) => None,
            Error::ParseIntError(ref e) => Some(e),
            Error::VarError(ref e) => Some(e),
            Error::IOError(ref e) => Some(e),
            Error::FormatError(ref e) => Some(e),
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Error {
        Error::FormatError(err)
    }
}
