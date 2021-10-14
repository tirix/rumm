use crate::parser::OptionalTactics;
use std::sync::Arc;
use crate::tactics::Tactics;
use crate::error::{Error, Result};
use crate::parser::Parser;
use crate::parser::Parse;
use crate::parser::Token;
use crate::tactics::{TacticsError, TacticsResult};
use crate::context::Context;
use metamath_knife::Formula;
use metamath_knife::Label;
use crate::lang::{Db, Display};
use core::fmt::Formatter;

/// An expression evaluating to a formula
pub enum FormulaExpression {
    Goal,
    Formula(Formula),
    Statement(StatementExpression),
}

impl Display for FormulaExpression {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        match self {
            FormulaExpression::Goal => fmt.write_str("goal"),
            FormulaExpression::Formula(f) => f.format(fmt, db),
            FormulaExpression::Statement(l) => {
                l.format(fmt, db)?;
                fmt.write_str(".stmt")
            },
        }
    }
}

impl Parse for FormulaExpression {
    fn parse(parser: &mut Parser) -> Result<Self> {
        match parser.next_token() {
            Some(Token::FormulaStart) => Ok(FormulaExpression::Formula(parser.parse_mm_formula()?)),
            Some(Token::GoalKeyword) => Ok(FormulaExpression::Goal),
            Some(Token::StatementKeyword) => Ok(FormulaExpression::Statement(StatementExpression::parse(parser)?)),
            Some(token) => Err(Error::parse_error(
                "A match target, either a formula, the 'goal keyword, or a label statement'.",
                token,
            )),
            None => Err(Error::unexpected_end_of_file(
                "A match target, either a formula, the 'goal keyword, or a label statement'.",
            )),
        }
	}
}

impl FormulaExpression {
    pub fn evaluate(&self, context: &Context) -> TacticsResult<Formula> {
        match self {
            FormulaExpression::Statement(e) => Ok(context.get_theorem_formulas(e.evaluate(context)?).ok_or(TacticsError::Error)?.0),
            FormulaExpression::Goal => Ok(context.goal().clone()),
            FormulaExpression::Formula(f) => Ok(f.clone()),
        }
    }
}

/// An expression evaluating to a tactics
pub enum TacticsExpression {
    Constant(Arc<dyn Tactics>),
    Variable(String),
}

impl Display for TacticsExpression {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        match self {
            TacticsExpression::Constant(t) => t.format(fmt, db),
            TacticsExpression::Variable(id) => fmt.write_str(id),
        }
    }
}

impl TacticsExpression {
	pub fn cst(tactics: Arc<dyn Tactics>) -> Result<OptionalTactics> {
		Ok(OptionalTactics::Some(TacticsExpression::Constant(tactics)))
	}

	pub fn evaluate(&self, context: &Context) -> TacticsResult<Arc<dyn Tactics>> {
		match self {
			TacticsExpression::Constant(t) => Ok(t.clone()),
			TacticsExpression::Variable(id) => context.get_tactics_variable(id.to_string()).ok_or(TacticsError::Error),
		}
	}

	pub fn execute(&self, context: &mut Context) -> TacticsResult {
		self.evaluate(&context)?.execute(context)
	}
}

/// An expression evaluating to a statement
pub enum StatementExpression {
    Constant(Label),
    Variable(String),
}

impl Display for StatementExpression {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        match self {
            StatementExpression::Constant(l) => l.format(fmt, db),
            StatementExpression::Variable(id) => fmt.write_str(id),
        }
    }
}

impl Parse for StatementExpression {
    fn parse(parser: &mut Parser) -> Result<Self> {
    	match parser.next_token() {
    		Some(Token::TheoremLabel(name)) => Ok(StatementExpression::Constant(parser.get_theorem_label(name)?)),
    		Some(Token::TheoremIdentifier(id)) => Ok(StatementExpression::Variable(id)),
            Some(token) => Err(Error::parse_error("A statement expression", token)),
            None => Err(Error::unexpected_end_of_file("A statement expression")),
    	}
    }
}

impl StatementExpression {
	pub fn evaluate(&self, context: &Context) -> TacticsResult<Label> {
		match self {
			StatementExpression::Constant(l) => Ok(*l),
			StatementExpression::Variable(id) => context.get_label_variable(id.to_string()).ok_or(TacticsError::Error),
		}
	}
}

///
pub enum Expression {
	Formula(FormulaExpression),
	Statement(StatementExpression),
	Tactics(TacticsExpression),
}

impl Expression {
	pub fn tactics(tactics: Arc<dyn Tactics>) -> Result<Option<Expression>> {
		Ok(Some(Expression::Tactics(TacticsExpression::Constant(tactics))))
	}
}