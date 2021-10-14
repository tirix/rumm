use crate::error::{Error, Result};
use crate::parser::{Parse, Parser, Token};

#[derive(Debug)]
pub enum ParameterDefinition {
    Tactics(String),
    Theorem(String),
}

impl Parse for ParameterDefinition {
    fn parse(parser: &mut Parser) -> Result<Self> {
        match parser.next_token() {
            Some(Token::TacticsIdentifier(id)) => Ok(ParameterDefinition::Tactics(id)),
            Some(Token::TheoremIdentifier(id)) => Ok(ParameterDefinition::Theorem(id)),
            Some(token) => Err(Error::parse_error("String constant", token)),
            None => Err(Error::unexpected_end_of_file("String constant")),
        }
    }
}
