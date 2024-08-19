use crate::lang::TacticsExpression;
use crate::context::Context;
use crate::error::Result;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser, OptionalTactics};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use crate::trace::Trace;
use core::fmt::Formatter;

/// A tactics which tries a list of tactics until one of them produces a proof.
///
pub struct Try {
    tactics: Vec<TacticsExpression>,
}

impl Display for Try {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Try \n")?;
        for t in &self.tactics {
            t.format(fmt, db)?;
        }
        fmt.write_str("}\n")
    }
}

impl Parse for Try {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let mut tactics = Vec::new();
        while let OptionalTactics::Some(t) = parser.parse_optional_tactics()? {
            tactics.push(t);
        }
        Ok(Try { tactics })
    }
}

impl Tactics for Try {
    fn get_name(&self) -> String {
        "try".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which tries a list of tactics until one of them produces a proof.".to_string()
    }

    fn execute_intern(&self, trace: &mut Trace, context: &mut Context) -> TacticsResult {
        for t in &self.tactics {
            match t.execute(trace, context) {
                Ok(step) => {
                    return Ok(step);
                },
                Err(e) => {
                    trace.message(format!("{:?}",e).as_str());
                },
            }
        }
        Err(TacticsError::NoMatchFound)
    }
}
