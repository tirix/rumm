use crate::error::{Error, Result};
use crate::lang::*;
use crate::script::Script;
use crate::tactics::*;
use logos::{Lexer, Logos};
use metamath_knife::Formula;
use metamath_knife::Label;
use std::fmt::Display;
use std::fmt::Formatter;

// TODO in order to provide line number errors, I think we would need to implement Logos::Source...
// OR we add a token for newlines, which we use to count the line, and the position of the last line start :)

//	#[regex(r"/\\*(?:[^\\*]|\\*[^/])*\\*/", logos::skip)] // Skip multi-line comments
//	#[regex(r"/\\*([^*]|\\*+[^\\*/])*\\*+/", logos::skip)] // Skip multi-line comments

#[derive(Logos, Clone, Debug, PartialEq)]
pub enum Token {
    #[token("tactics")]
    TacticsKeyword,

    #[token("proof")]
    ProofKeyword,

    #[token("import")]
    ImportKeyword,

    #[token("load")]
    LoadKeyword,

    #[token("?")]
    TodoKeyword,

    #[token("!")]
    HypoKeyword,

    #[regex("\"[^\"]*\"", |lexer| {
	    let mut chars = lexer.slice().chars();
		chars.next(); // remove first char (apostrophe)
		chars.next_back(); // remove last char (apostrophe)
		String::from(chars.as_str())
		})]
    StringConstant(String),

    #[token("{")]
    CurlyBracketOpen,

    #[token("}")]
    CurlyBracketClose,

    #[token("(")]
    ParensOpen,

    #[token(")")]
    ParensClose,

    #[token("$")]
    FormulaStart,

    #[regex("~[a-zA-Z0-9.\\-_]+", |lexer| String::from(lexer.slice()))]
    TheoremLabel(String),

    #[regex("[a-zA-Z0-9.\\-_]+", |lexer| String::from(lexer.slice()))]
    Identifier(String),

    #[regex(r"/\*\*([^\*]|\*[^/])+\*/", |lexer| String::from(lexer.slice()))]
    // Descriptive comments
    DescriptiveComment(String),

    #[error]
    #[regex(r"[ \t\r\n]+", logos::skip)] // Skip spaces and line breaks
    #[regex(r"//[^\r\n]*(\r\n|\n)?", logos::skip)] // Skip line comments
    #[regex(r"/\*([^\*]|\*[^/])+\*/", logos::skip)] // Skip multi-line comments
    LexerError,
}

#[derive(Logos, Clone, Debug, PartialEq)]
enum MMToken {
    #[regex(r"[!-~]+", |lexer| String::from(lexer.slice()))]
    Token(String),

    #[regex(r"&[!-~]+", |lexer| String::from(lexer.slice()))]
    Variable(String),

    #[token("$")]
    End,

    #[error]
    #[regex(r"[ \t\r\n]+", logos::skip)] // Skip spaces and line breaks
    LexerError,
}

impl Display for Token {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

pub struct Parser<'a> {
    pub lexer: Lexer<'a, Token>,
    last_description: Option<String>,
    db: Db,
    debug: bool,
}

impl<'a> Parser<'a> {
    pub fn new(str: &'a str) -> Self {
        Parser {
            lexer: Token::lexer(str),
            last_description: None,
            db: Db::default(),
            debug: false,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let mut token = self.lexer.next();
        while let Some(Token::DescriptiveComment(description)) = token {
            self.last_description = Some(description);
            token = self.lexer.next();
        }
        if self.debug {
            println!("\tToken: {:?} ({:?})", token, self.lexer.slice());
        }
        token
    }

    pub fn last_description(&mut self) -> Option<String> {
        let last_desscription = self.last_description.clone();
        self.last_description = None;
        last_desscription
    }

    pub fn parse_token(&mut self, expected: Token) -> Result {
        match self.next_token() {
            Some(found) => {
                if found == expected {
                    Ok(())
                } else {
                    Err(Error::ParseError {
                        expected: expected.to_string(),
                        found,
                    })
                }
            }
            None => Err(Error::unexpected_end_of_file("Identifier")),
        }
    }

    pub fn parse_curly_bracket_close(&mut self) -> Result {
        self.parse_token(Token::CurlyBracketClose)
    }

    pub fn parse_string_constant(&mut self) -> Result<String> {
        match self.next_token() {
            Some(Token::StringConstant(string)) => Ok(string),
            Some(token) => Err(Error::parse_error("String constant", token)),
            None => Err(Error::unexpected_end_of_file("String constant")),
        }
    }

    pub fn parse_identifier(&mut self) -> Result<String> {
        match self.next_token() {
            Some(Token::Identifier(name)) => Ok(name),
            Some(token) => Err(Error::parse_error("Identifier", token)),
            None => Err(Error::unexpected_end_of_file("Identifier")),
        }
    }

    pub fn parse_theorem_label(&mut self) -> Result<Label> {
        match self.next_token() {
            Some(Token::TheoremLabel(name)) => self.db.get_theorem_label(name[1..].to_string()),
            Some(token) => Err(Error::parse_error("A Theorem Label", token)),
            None => Err(Error::unexpected_end_of_file("A Theorem Label")),
        }
    }

    pub fn parse_parameter_definition(&mut self) -> Result<Option<ParameterDefinition>> {
        let definition_result = ParameterDefinition::parse(self);
        match definition_result {
            Ok(parameter_definition) => Ok(Some(parameter_definition)),
            Err(Error::ParseError {
                expected: _,
                found: Token::ParensClose,
            }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn parse_tactics(&mut self) -> Result<Box<dyn Tactics>> {
        match self.parse_optional_tactics() {
            Ok(Some(tactics)) => Ok(tactics),
            Ok(None) => Err(Error::parse_error(
                "A tactics, within braces '{ ... }', or the '?' unfinished tactics.",
                Token::CurlyBracketClose,
            )),
            Err(error) => Err(error),
        }
    }

    pub fn parse_optional_tactics(&mut self) -> Result<Option<Box<dyn Tactics>>> {
        match self.next_token() {
            Some(Token::CurlyBracketClose) => Ok(None),
            Some(Token::TodoKeyword) => Ok(Some(Box::new(Skipped {}))),
            Some(Token::HypoKeyword) => Ok(Some(Box::new(Hypothesis {}))),
            Some(Token::CurlyBracketOpen) => match self.next_token() {
                Some(Token::Identifier(name)) => match name.as_ref() {
                    "use" => Ok(Some(UseScriptTactics::parse(self)?.boxed())),
                    "subgoal" => Ok(Some(Subgoal::parse(self)?.boxed())),
                    "apply" => Ok(Some(Apply::parse(self)?.boxed())),
                    "try" => Ok(Some(Try::parse(self)?.boxed())),
                    "match" => Ok(Some(Match::parse(self)?.boxed())),
                    _ => Err(Error::msg(format!("Unknown tactics name: {}", name))),
                },
                Some(token) => Err(Error::parse_error("A tactics name", token)),
                None => Err(Error::unexpected_end_of_file("A tactics name")),
            },
            Some(token) => Err(Error::parse_error(
                "A tactics, within braces '{ ... }', or the '?' unfinished tactics.",
                token,
            )),
            None => Err(Error::unexpected_end_of_file(
                "A tactics, within braces '{ ... }', or the '?' unfinished tactics.",
            )),
        }
    }

    pub fn parse_formula(&mut self) -> Result<Formula> {
        match self.parse_optional_formula() {
            Ok(Some(formula)) => Ok(formula),
            Ok(None) => Err(Error::parse_error(
                "A formula, within dollar signs '$ ... $'.",
                Token::CurlyBracketClose,
            )),
            Err(error) => Err(error),
        }
    }

    pub fn parse_optional_formula(&mut self) -> Result<Option<Formula>> {
        match self.next_token() {
            Some(Token::CurlyBracketClose) => Ok(None),
            Some(Token::FormulaStart) => {
                let mut mmlex = self.lexer.to_owned().morph();
                let mut symbols = Vec::new();
                for t in &mut mmlex {
                    match t {
                        MMToken::Token(name) => {
                            symbols.push(self.db.get_symbol(name)?);
                        }
                        MMToken::Variable(name) => {
                            symbols.push(self.db.get_symbol(name)?);
                        }
                        MMToken::End => {
                            break;
                        }
                        MMToken::LexerError => {
                            return Err(Error::MMLexerError);
                        }
                    }
                }
                self.lexer = mmlex.morph();
                Ok(Some(self.db.parse_formula(symbols)?))
            }
            Some(token) => Err(Error::parse_error(
                "A formula, within dollar signs '$ ... $'.",
                token,
            )),
            None => Err(Error::unexpected_end_of_file(
                "A formula, within dollar signs '$ ... $'.",
            )),
        }
    }

    pub fn parse_script(&mut self) -> Result<Script> {
        let mut tactics_definitions = Vec::new();
        let mut proof_definitions = Vec::new();
        loop {
            match self.next_token() {
                Some(Token::LoadKeyword) => {
                    let filename = self.parse_string_constant()?;
                    self.db.parse(filename)?;
                }
                Some(Token::TacticsKeyword) => {
                    tactics_definitions.push(TacticsDefinition::parse(self)?);
                }
                Some(Token::ProofKeyword) => {
                    proof_definitions.push(ProofDefinition::parse(self)?);
                }
                Some(token) => {
                    return Err(Error::parse_error(
                        "The tactics or the proof keywords",
                        token,
                    ));
                }
                None => {
                    return Ok(Script::new(
                        self.db.clone(),
                        tactics_definitions,
                        proof_definitions,
                    ));
                }
            }
        }
    }
}

pub trait Parse {
    fn parse(parser: &mut Parser) -> Result<Self>
    where
        Self: Sized;
}
