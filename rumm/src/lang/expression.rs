use crate::parser::FormulaOrSubstitutionListId;
use crate::parser::OptionalTactics;
use std::sync::Arc;
use crate::tactics::Tactics;
use crate::error::Result;
use crate::parser::Parser;
use crate::parser::Parse;
use crate::parser::Token;
use crate::tactics::{TacticsError, TacticsResult};
use crate::context::Context;
use metamath_knife::Formula;
use metamath_knife::Label;
use metamath_knife::formula::Substitutions;
use crate::lang::{Db, Display};
use core::fmt::Formatter;

/// An expression evaluating to a formula
pub enum FormulaExpression {
    Goal,
    Formula(Formula),
    Variable(String),
    Statement(StatementExpression),
    DirectSubstitution(Formula, Box<FormulaExpression>, Box<FormulaExpression>),
    ListSubstitution(String, Box<FormulaExpression>),
}

impl Display for FormulaExpression {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        match self {
            FormulaExpression::Goal => fmt.write_str("goal"),
            FormulaExpression::Formula(f) => f.format(fmt, db),
            FormulaExpression::Variable(id) => fmt.write_str(id),
            FormulaExpression::Statement(l) => {
                l.format(fmt, db)?;
                fmt.write_str("statement")
            },
            FormulaExpression::DirectSubstitution(what, with, in_expr) => {
                fmt.write_str("s/")?;
                what.format(fmt, db)?;
                fmt.write_str("/")?;
                with.format(fmt, db)?;
                fmt.write_str("/")?;
                in_expr.format(fmt, db)
            }
            FormulaExpression::ListSubstitution(id, in_expr) => {
                fmt.write_fmt(format_args!("s/ *{id} /"))?;
                in_expr.format(fmt, db)
            }
        }
    }
}

impl Parse for FormulaExpression {
    fn parse(parser: &mut Parser) -> Result<Self> {
        match parser.next_token() {
            Some(Token::FormulaStart) => Ok(FormulaExpression::Formula(parser.parse_mm_formula()?)),
            Some(Token::GoalKeyword) => Ok(FormulaExpression::Goal),
            Some(Token::StatementKeyword) => Ok(FormulaExpression::Statement(StatementExpression::parse(parser)?)),
    		Some(Token::FormulaIdentifier(id)) => Ok(FormulaExpression::Variable(id)),
            Some(Token::BeginSubstitutionKeyword) => {
                match parser.parse_formula_or_substvar()? {
                    FormulaOrSubstitutionListId::Formula(substitute_what) => {
                        parser.parse_token(Token::SubstitutionKeyword)?;
                        let substitute_with = parser.parse_formula_expression()?;
                        parser.parse_token(Token::SubstitutionKeyword)?;
                        let substitute_in = parser.parse_formula_expression()?;
                        Ok(FormulaExpression::DirectSubstitution(substitute_what, Box::new(substitute_with), Box::new(substitute_in)))
                    },
                    FormulaOrSubstitutionListId::SubstitutionListIdentifier(id) => {
                        parser.parse_token(Token::SubstitutionKeyword)?;
                        let substitute_in = parser.parse_formula_expression()?;
                        Ok(FormulaExpression::ListSubstitution(id, Box::new(substitute_in)))
                    }
                }
            },
            Some(token) => Err(parser.parse_error(
                "A match target, either a formula, the 'goal keyword, or a label statement'.",
                token,
            ).into()),
            None => Err(parser.unexpected_end_of_file(
                "A match target, either a formula, the 'goal keyword, or a label statement'.",
            ).into()),
        }
	}
}

impl FormulaExpression {
    pub fn evaluate(&self, context: &Context) -> TacticsResult<Formula> {
        match self {
            FormulaExpression::Statement(e) => { let label = e.evaluate(context)?; Ok(context.get_theorem_formulas(label).ok_or(TacticsError::UnknownLabel(label))?.0) },
            FormulaExpression::Goal => Ok(context.goal().clone()),
            FormulaExpression::Formula(f) => Ok(f.clone()),
            FormulaExpression::Variable(id) => context.get_formula_variable(id.to_string()).ok_or(TacticsError::UnknownFormulaVariable(id.to_string())),
            FormulaExpression::DirectSubstitution(what, with, in_expr) => Ok(in_expr.evaluate(context)?.substitute(context.variables()).replace(&what.substitute(context.variables()), &with.evaluate(context)?.substitute(context.variables()))),
            FormulaExpression::ListSubstitution(id, in_expr) => Ok(in_expr.evaluate(context)?.substitute(context.variables()).substitute(context.get_substitution_variable(id.to_string()).ok_or(TacticsError::UnknownSubstitutionVariable(id.to_string()))?))
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
			TacticsExpression::Variable(id) => context.get_tactics_variable(id.to_string()).ok_or(TacticsError::UnknownTacticsVariable(id.to_string())),
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
            Some(token) => Err(parser.parse_error("A statement expression", token).into()),
            None => Err(parser.unexpected_end_of_file("A statement expression").into()),
    	}
    }
}

impl StatementExpression {
	pub fn evaluate(&self, context: &Context) -> TacticsResult<Label> {
		match self {
			StatementExpression::Constant(l) => Ok(*l),
			StatementExpression::Variable(id) => context.get_label_variable(id.to_string()).ok_or(TacticsError::UnknownLabelVariable(id.to_string())),
		}
	}
}

// An expression evaluating to a list of substitutions
#[derive(Default)]
pub struct SubstitutionListExpression {
    list: Vec<SubstitutionExpression>,
}

pub enum SubstitutionExpression {
    Constant((Label, FormulaExpression)),
    Variable(String),
}

impl Display for SubstitutionListExpression {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        for subst in &self.list {
            match subst {
                SubstitutionExpression::Constant((l, f)) => {
                    l.format(fmt, db)?; 
                    fmt.write_str(" ")?; 
                    f.format(fmt, db)?;
                },
                SubstitutionExpression::Variable(id) => fmt.write_str(&id)?,
            }
        }
        Ok(())
    }
}

impl Parse for SubstitutionListExpression {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let mut list = vec![];
        while let Some(subst) = parser.parse_substitution_expression()? {
            list.push(subst);
        }
        Ok(Self{list})
    }
}

impl SubstitutionListExpression {
	pub fn evaluate(&self, context: &Context) -> TacticsResult<Substitutions> {
        let mut subst = Substitutions::new();
        for s in &self.list {
            match s {
                SubstitutionExpression::Constant((l, f)) => { subst.insert(*l, context.db.ensure_type(f.evaluate(context)?, *l)?); },
                SubstitutionExpression::Variable(id) => { subst.extend(context.get_substitution_variable(id.to_string()).ok_or(TacticsError::UnknownSubstitutionVariable(id.to_string()))?); },
            }
        }
        TacticsResult::Ok(subst)
	}
}

///
pub enum Expression {
	Formula(FormulaExpression),
	Statement(StatementExpression),
	Tactics(TacticsExpression),
    SubstitutionList(SubstitutionListExpression),
}

impl Expression {
	pub fn tactics(tactics: Arc<dyn Tactics>) -> Result<Option<Expression>> {
		Ok(Some(Expression::Tactics(TacticsExpression::Constant(tactics))))
	}
}