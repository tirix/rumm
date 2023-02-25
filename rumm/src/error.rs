use crate::parser::Token;
use annotate_snippets::{snippet::{Snippet, Annotation, AnnotationType, Slice, SourceAnnotation}, display_list::FormatOptions};
use metamath_knife::{diag::{Diagnostic, StmtParseError}, statement::StatementAddress};
use typed_arena::Arena;



// Check if crate "thiserror" would help here?

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Location {
    pub filename: String,
    pub line: String,
    pub line_number: usize,
    pub start: usize,
    pub end: usize,
}

impl Location {
    fn to_slice<'a>(&self, label: String, arena: &'a Arena<String>) -> Slice<'a> {
        Slice{
            source: arena.alloc(self.line.clone()),
            line_start: self.line_number,
            origin: Some(arena.alloc(self.filename.clone())),
            annotations: vec![SourceAnnotation{
                range: (self.start, self.end),
                label: arena.alloc(label.clone()),
                annotation_type: AnnotationType::Note }],
            fold: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Metamath Error {} while parsing {0}", Error::format_mm_err(.1))]
    DBError(String, Vec<(StatementAddress, Diagnostic)>),
    #[error("Metamath Parse Error ")]
    DBParseError { #[source] error: StmtParseError, location: Location },
    #[error("Lexer Error")]
    MMLexerError { location: Location },
    #[error("While parsing parameters for tactics {1}")]
    TacticsParameterParseError (#[source] Box<Error>, String),
    #[error("While parsing tactics {1}")]
    TacticsParseError (#[source] Box<Error>, String),
    #[error("Parse Error: expected {expected}, found {found}")]
    ParseError { location: Location, expected: String, found: Token },
    #[error("Unexpected end of file: expected {expected}")]
    UnexpectedEndOfFile { location: Location, expected: String },
    #[error("Unknown symbol {symbol}")]
    UnknownSymbol{ location: Location, symbol: String },
    #[error("Unknown label {label}")]
    UnknownLabel{ location: Location, label: String },
    #[error("Unknown tactics name {1}")]
    UnknownTacticsName(Location, String),
}

impl Error {
    pub fn format_mm_err(data: &Vec<(StatementAddress, Diagnostic)>) -> String {
        data.iter().fold(String::new(), |s, (saddr, diag)| format!("{}\n{:?}:{:?}", s, saddr, diag))
    }

    pub fn caused_by(&self) -> Option<&Error> {
        match self {
            Error::TacticsParameterParseError(e, _) => Some(&*e),
            Error::TacticsParseError(e, _) => Some(&*e),
            _ => None,
        }
    }

    fn location(&self) -> &Location {
        match self {
            Error::IoError(_) => todo!(),
            Error::DBError(_, _) => todo!(),
            Error::DBParseError { location, .. } => location,
            Error::MMLexerError { location } => location,
            Error::TacticsParameterParseError(e, _) => e.location(),
            Error::TacticsParseError(e, _) => e.location(),
            Error::ParseError { location, .. } => location,
            Error::UnexpectedEndOfFile { location, .. } => location,
            Error::UnknownSymbol { location, .. } => location,
            Error::UnknownLabel { location, .. } => location,
            Error::UnknownTacticsName(location, _) => location,
        }
    }

    fn add_slice<'a>(&self, slices: &mut Vec<Slice<'a>>, arena: &'a Arena<String>) {
        let location = self.location();
        slices.push(location.to_slice(format!("{}",self), arena));
        self.caused_by().and_then(|e| Some(e.add_slice(slices, arena)));
    }

    pub fn to_snippet<'a>(&self, arena: &'a Arena<String>) -> Snippet<'a> {
        let mut slices = vec![];
        self.add_slice(&mut slices, arena);
        Snippet {
            title: Some(Annotation{
                id: None,
                label: Some(arena.alloc(format!("{}",self))), 
                annotation_type: AnnotationType::Error,
            }),
            footer: vec![],
            slices,
            opt: FormatOptions {
                color: true,
                ..FormatOptions::default()
            },
        }
    }
}