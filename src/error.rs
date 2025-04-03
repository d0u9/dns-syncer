use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HttpError(String),
    ParseError(String),
    IoError(std::io::Error),
    GlobalFetcherError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HttpError(e) => write!(f, "HTTP error: {}", e),
            Error::ParseError(e) => write!(f, "Parse error: {}", e),
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::GlobalFetcherError(e) => write!(f, "Global fetcher error: {}", e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::HttpError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(err: std::net::AddrParseError) -> Error {
        Error::ParseError(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::ParseError(err.to_string())
    }
}
