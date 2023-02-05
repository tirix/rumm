use crate::parser::Token;
use metamath_knife::{diag::Diagnostic, statement::StatementAddress, formula::UnificationError};

// Check if crate "thiserror" would help here?

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DBError(Vec<(StatementAddress, Diagnostic)>),
    ParseError { expected: String, found: Token },
    MMLexerError,
    MessageError(String),
    UnexpectedEndOfFile { expected: String },
}

impl Error {
    pub fn parse_error(expected: &str, found: Token) -> Self {
        Error::ParseError {
            expected: expected.to_string(),
            found,
        }
    }

    pub fn unexpected_end_of_file(expected: &str) -> Self {
        Error::UnexpectedEndOfFile {
            expected: expected.to_string(),
        }
    }

    pub fn msg<T: ToString>(msg: T) -> Self {
        Error::MessageError(msg.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::msg(format!("io error: {}", error))
    }
}

impl From<UnificationError> for Error {
    fn from(_: UnificationError) -> Self {
        Error::msg(format!("Unification failed"))
    }
}

pub struct Backtrace {}
