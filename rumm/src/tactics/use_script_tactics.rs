use crate::lang::Expression;
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
    parameters: Vec<Expression>,
}

impl Parse for UseScriptTactics {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let name = parser.parse_identifier()?;
        let parameters = parser.parse_parameters()?;
        Ok(UseScriptTactics { name, parameters })
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
        println!("-- Use --");
        if let Some(tactics_definition) = context.clone().get_tactics_definition(self.name.clone())
        {
            tactics_definition.add_variables(context, &self.parameters)?;
            tactics_definition.execute(context)
        } else {
            Err(TacticsError::Error)
        }
    }
}
