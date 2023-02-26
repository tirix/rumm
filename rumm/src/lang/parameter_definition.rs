
use crate::error::Result;
use crate::parser::{Parse, Parser, Token};

#[derive(Debug)]
pub enum ParameterDefinition {
    Tactics(String),
    Theorem(String),
    Formula(String),
    SubstitutionList(String),
}

impl Parse for ParameterDefinition {
    fn parse(parser: &mut Parser) -> Result<Self> {
        match parser.next_token() {
            Some(Token::TacticsIdentifier(id)) => Ok(ParameterDefinition::Tactics(id)),
            Some(Token::TheoremIdentifier(id)) => Ok(ParameterDefinition::Theorem(id)),
            Some(Token::FormulaIdentifier(id)) => Ok(ParameterDefinition::Formula(id)),
            Some(Token::WithKeyword) => Self::parse_substitution_parameter_definition(parser),
            Some(token) => Err(parser.parse_error("String constant", token).into()),
            None => Err(parser.unexpected_end_of_file("String constant").into()),
        }
    }
}

impl ParameterDefinition {
    fn parse_substitution_parameter_definition(parser: &mut Parser) -> Result<Self> {
        match parser.next_token() {
            Some(Token::SubstitutionListIdentifier(id)) => Ok(ParameterDefinition::SubstitutionList(id)),
            Some(token) => Err(parser.parse_error("A substitution variable identifier starting with *", token).into()),
            None => Err(parser.unexpected_end_of_file("A substitution variable identifier starting with *").into()),
        }
    }
}