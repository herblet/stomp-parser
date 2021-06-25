//! Defines error types used and returned by this package.
use std::convert::Into;

use nom::error::{ContextError, FromExternalError, ParseError, VerboseError};

#[derive(Debug)]
pub struct StompParseError {
    message: String,
}

impl StompParseError {
    pub fn new<S: Into<String>>(message: S) -> StompParseError {
        StompParseError {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<std::io::Error> for StompParseError {
    fn from(io_error: std::io::Error) -> Self {
        StompParseError::new(format!("IO-Error writing item: {:?}", io_error))
    }
}

pub trait FullError<I, E>: ParseError<I> + FromExternalError<I, E> + ContextError<I> {}

impl<I, E> FullError<I, E> for VerboseError<I> {}
