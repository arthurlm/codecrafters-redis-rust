use std::{io, num::ParseIntError, str::Utf8Error, string::FromUtf8Error};

use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum MiniRedisError {
    #[error("I/O: {0}")]
    Io(String),

    #[error("Invalid text: {0}")]
    InvalidText(String),

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(char),

    #[error("Invalid message end")]
    InvalidMessageEnd,

    #[error("Invalid RDB magic number")]
    InvalidRdbMagicNumber,

    #[error("Unsupported length encoding")]
    UnsupportedLengthEncoding,
}

impl From<io::Error> for MiniRedisError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<FromUtf8Error> for MiniRedisError {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidText(err.to_string())
    }
}

impl From<Utf8Error> for MiniRedisError {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidText(err.to_string())
    }
}

impl From<ParseIntError> for MiniRedisError {
    fn from(err: ParseIntError) -> Self {
        Self::InvalidNumber(err.to_string())
    }
}
