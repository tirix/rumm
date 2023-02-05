use crate::lang::TacticsExpression;
use crate::error::{Error, Result};
use crate::lang::*;
use crate::script::Script;
use crate::tactics::*;
use logos::{Lexer, Logos};
use metamath_knife::{Formula, Span};
use metamath_knife::Label;
use metamath_knife::grammar::FormulaToken;
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

    #[token("goal")]
    GoalKeyword,

    #[token("statement")]
    StatementKeyword,

    #[token("with")]
    WithKeyword,

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

    #[regex("≈[a-zA-Z0-9.\\-_]+", |lexer| String::from(lexer.slice()))]
    TheoremIdentifier(String),

    #[regex("@[a-zA-Z0-9.\\-_]+", |lexer| String::from(lexer.slice()))]
    TacticsIdentifier(String),

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

pub enum OptionalTactics {
    Some(TacticsExpression),
    With,
    None
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
            debug: true,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let mut token = self.lexer.next();
        while let Some(Token::DescriptiveComment(description)) = token {
            self.last_description = Some(description);
            token = self.lexer.next();
        }
        if self.debug && false {
            println!("\tToken: {:?} ({:?})", token, self.lexer.slice());
        }
        token
    }

    pub fn last_description(&mut self) -> Option<String> {
        let last_desscription = self.last_description.clone();
        self.last_description = None;
        last_desscription
    }

    pub fn get_theorem_label(&self, name: String) -> Result<Label> {
        self.db.get_theorem_label(name[1..].to_string())
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

    pub fn parse_optional_statement(&mut self) -> Result<Option<Label>> {
        match self.next_token() {
            Some(Token::CurlyBracketClose) => Ok(None),
            Some(Token::TheoremLabel(name)) => Ok(Some(self.db.get_theorem_label(name[1..].to_string())?)),
            Some(token) => Err(Error::parse_error("A Theorem Label", token)),
            None => Err(Error::unexpected_end_of_file("A Theorem Label")),
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

    pub fn parse_tactics(&mut self) -> Result<TacticsExpression> {
        match self.parse_optional_tactics() {
            Ok(OptionalTactics::Some(tactics)) => Ok(tactics),
            Ok(OptionalTactics::None) | Ok(OptionalTactics::With) => Err(Error::parse_error(
                "A tactics, within braces '{ ... }', or the '?' unfinished tactics.",
                Token::CurlyBracketClose,
            )),
            Err(error) => Err(error),
        }
    }

    pub fn parse_optional_tactics(&mut self) -> Result<OptionalTactics> {
        match self.next_token() {
            Some(Token::CurlyBracketClose) => Ok(OptionalTactics::None),
            Some(Token::TacticsIdentifier(id)) => Ok(OptionalTactics::Some(TacticsExpression::Variable(id))),
            Some(Token::TodoKeyword) => TacticsExpression::cst(Skipped {}.into_arc()),
            Some(Token::HypoKeyword) => TacticsExpression::cst(Hypothesis {}.into_arc()),
            Some(Token::CurlyBracketOpen) => match self.next_token() {
                Some(Token::Identifier(name)) => match name.as_ref() {
                    "use" => TacticsExpression::cst(UseScriptTactics::parse(self)?.into_arc()),
                    "subgoal" => TacticsExpression::cst(Subgoal::parse(self)?.into_arc()),
                    "apply" => TacticsExpression::cst(Apply::parse(self)?.into_arc()),
                    "try" => TacticsExpression::cst(Try::parse(self)?.into_arc()),
                    "match" => TacticsExpression::cst(Match::parse(self)?.into_arc()),
                    "find" => TacticsExpression::cst(Find::parse(self)?.into_arc()),
                    _ => Err(Error::msg(format!("Unknown tactics name: {}", name))),
                },
                Some(token) => Err(Error::parse_error("A tactics name", token)),
                None => Err(Error::unexpected_end_of_file("A tactics name")),
            },
            Some(Token::WithKeyword) => Ok(OptionalTactics::With),
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
                Ok(Some(self.parse_mm_formula()?))
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

    pub fn parse_mm_formula(&mut self) -> Result<Formula> {
        let mut mmlex = self.lexer.to_owned().morph();
        let mut symbols = Vec::new();
        for t in &mut mmlex {
            let span = Span::new(0, 1); // We could use mmlex.spanned() to get the spans of each token!
            match t {
                MMToken::Token(name) => {
                    symbols.push(FormulaToken { symbol: self.db.get_symbol(name)?, span });
                }
                MMToken::Variable(name) => {
                    symbols.push(FormulaToken { symbol: self.db.get_symbol(name)?, span });
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
        Ok(self.db.parse_formula(symbols)?)
    }

    pub fn parse_formula_expression(&mut self) -> Result<FormulaExpression> {
        FormulaExpression::parse(self)
    }

    pub fn parse_parameters(&mut self) -> Result<Vec<Expression>> {
        let mut parameters = Vec::new();
        loop {
            if let Some(p) = self.parse_optional_parameter()? {
                parameters.push(p);
            } else {
                return Ok(parameters);
            }
        }
    }

    pub fn parse_optional_parameter(&mut self) -> Result<Option<Expression>> {
        match self.next_token() {
            Some(Token::TheoremIdentifier(id)) => Ok(Some(Expression::Statement(StatementExpression::Variable(id)))),
            Some(Token::TacticsIdentifier(id)) => Ok(Some(Expression::Tactics(TacticsExpression::Variable(id)))),
            Some(Token::TheoremLabel(name)) => Ok(Some(Expression::Statement(StatementExpression::Constant(self.db.get_theorem_label(name[1..].to_string())?)))),
            Some(Token::TodoKeyword) => Expression::tactics(Skipped {}.into_arc()),
            Some(Token::HypoKeyword) => Expression::tactics(Hypothesis {}.into_arc()),
            Some(Token::CurlyBracketOpen) => match self.next_token() {
                Some(Token::Identifier(name)) => match name.as_ref() {
                    "use" => Expression::tactics(UseScriptTactics::parse(self)?.into_arc()),
                    "subgoal" => Expression::tactics(Subgoal::parse(self)?.into_arc()),
                    "apply" => Expression::tactics(Apply::parse(self)?.into_arc()),
                    "try" => Expression::tactics(Try::parse(self)?.into_arc()),
                    "match" => Expression::tactics(Match::parse(self)?.into_arc()),
                    "find" => Expression::tactics(Find::parse(self)?.into_arc()),
                    _ => Err(Error::msg(format!("Unknown tactics name: {}", name))),
                },
                Some(token) => Err(Error::parse_error("A tactics name", token)),
                None => Err(Error::unexpected_end_of_file("A tactics name")),
            },
            Some(Token::CurlyBracketClose) => Ok(None),
            Some(token) => Err(Error::parse_error(
                "The tactics or the proof keywords",
                token,
            ))?,
            None => Err(Error::unexpected_end_of_file(
                "The tactics or the proof keywords",
            ))?,
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
