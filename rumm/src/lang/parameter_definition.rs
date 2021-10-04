use crate::error::{Error, Result};
use crate::parser::{Parse, Parser};

#[derive(Debug)]
pub enum ParameterDefinition {
    Tactics(String),
}

impl Parse for ParameterDefinition {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.parse_identifier()?;
        Err(Error::msg("Not yet implemented"))
    }
}
