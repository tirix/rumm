use crate::context::Context;
use crate::error::Result;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use crate::trace::Trace;
use core::fmt::Formatter;

/// A special tactics, representing a proof not completed yet, marked by a question mark.
///
#[derive(Debug)]
pub struct Skipped;

impl Parse for Skipped {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.parse_curly_bracket_close()?;
        Ok(Skipped {})
    }
}

impl Display for Skipped {
    fn format(&self, fmt: &mut Formatter, _db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("?")
    }
}

impl Tactics for Skipped {
    fn get_name(&self) -> String {
        "?".to_string()
    }

    fn get_desc(&self) -> String {
        "The \"to do\" tactics, leaving the goal unproven and filling in the Metamath proof with an incomplete, question mark proof.".to_string()
    }

    fn execute_intern(&self, _trace: &mut Trace, _context: &mut Context) -> TacticsResult {
        Err(TacticsError::Skipped)
    }
}
