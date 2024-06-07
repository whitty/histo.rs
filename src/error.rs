// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

#[derive(Debug)]
pub enum Error {
    NoData,
    VarError(std::env::VarError),
    ParseIntError(std::num::ParseIntError),
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
            Error::NoData => None,
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
