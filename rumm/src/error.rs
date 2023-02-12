use crate::parser::Token;
use logos::Span;
use metamath_knife::{diag::{Diagnostic, StmtParseError}, statement::StatementAddress, formula::UnificationError};



// Check if crate "thiserror" would help here?

pub type Result<T = ()> = anyhow::Result<T, anyhow::Error>;

#[derive(Debug)]
pub struct Location {
    pub filename: String,
    pub span: Span,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Metamath Error {}", Error::format_mm_err(.0))]
    DBError(Vec<(StatementAddress, Diagnostic)>),
    #[error("Metamath Parse Error ")]
    DBParseError(StmtParseError),
    #[error("Lexer Error")]
    MMLexerError,
    #[error("Parse Error: expected {expected}, found {found}")]
    ParseError { location: Location, expected: String, found: Token },
    #[error("Unification Failed")]
    UnificationError(UnificationError),
    #[error("Unexpected end of file: expected {expected}")]
    UnexpectedEndOfFile { location: Location, expected: String },
    #[error("Unknown label {label}")]
    UnknownLabel{ location: Location, label: String },
}

impl Error {
    pub fn format_mm_err(data: &Vec<(StatementAddress, Diagnostic)>) -> String {
        data.iter().fold(String::new(), |s, (saddr, diag)| format!("{}\n{:?}:{:?}", s, saddr, diag))
    }
}