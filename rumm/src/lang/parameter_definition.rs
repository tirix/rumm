use crate::error::Result;
use crate::parser::{Parse, Parser, Token};

#[derive(Debug)]
pub enum ParameterDefinition {
    Tactics(String),
    Theorem(String),
    Formula(String),
}

impl Parse for ParameterDefinition {
    fn parse(parser: &mut Parser) -> Result<Self> {
        match parser.next_token() {
            Some(Token::TacticsIdentifier(id)) => Ok(ParameterDefinition::Tactics(id)),
            Some(Token::TheoremIdentifier(id)) => Ok(ParameterDefinition::Theorem(id)),
            Some(Token::FormulaIdentifier(id)) => Ok(ParameterDefinition::Formula(id)),
            Some(token) => Err(parser.parse_error("String constant", token).into()),
            None => Err(parser.unexpected_end_of_file("String constant").into()),
        }
    }
}
