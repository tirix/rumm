use crate::context::Context;
use crate::error::Result;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;

/// Calling a script tactics.
///
pub struct UseScriptTactics {
    name: String,
    // parameters: Vec<Value>,
}

impl Parse for UseScriptTactics {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let name = parser.parse_identifier()?;
        // TODO : parse parameters
        parser.parse_curly_bracket_close()?;
        Ok(UseScriptTactics { name })
    }
}

impl Display for UseScriptTactics {
    fn format(&self, fmt: &mut Formatter, _db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Match \n")?;
        fmt.write_str(&self.name)?;
        fmt.write_str("}\n")
    }
}

impl Tactics for UseScriptTactics {
    fn get_name(&self) -> String {
        format!("use {}", self.name)
    }

    fn get_desc(&self) -> String {
        "A tactics for calling a tactics defined in the Rumm script.".to_string()
    }

    fn execute(&self, context: &mut Context) -> TacticsResult {
        if let Some(tactics_definition) = context.clone().get_tactics_definition(self.name.clone())
        {
            // Set parameters as variables in the context...

            tactics_definition.execute(context)
        } else {
            Err(TacticsError::Error)
        }
    }
}
