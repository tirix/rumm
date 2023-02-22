use metamath_knife::formula::Substitutions;
use metamath_knife::Formula;
use crate::lang::DisplayPair;
use crate::lang::FormulaExpression;
use crate::lang::TacticsExpression;
use crate::context::Context;
use crate::error::Result;
use crate::lang::ProofStep;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;


/// A tactics which matches the goal with a hypothesis.
///
pub struct FindHyp {
    formula: FormulaExpression,
    tactics: TacticsExpression,
}

impl Display for FindHyp {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Find! \n")?;
        self.formula.format(fmt, db)?;
        self.tactics.format(fmt, db)?;
        fmt.write_str("}\n")
    }
}

impl Parse for FindHyp {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let formula = FormulaExpression::parse(parser)?;
        let tactics = parser.parse_tactics()?;
        parser.parse_curly_bracket_close()?;
        Ok(FindHyp {
            formula,
            tactics,
        })
    }
}

impl Tactics for FindHyp {
    fn get_name(&self) -> String {
        "find!".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which searches for a hypothesis matching the given formula.".to_string()
    }

    fn execute(&self, context: &mut Context) -> TacticsResult {
        let target = self.formula.evaluate(context)?.substitute(context.variables());
        context.enter(&format!("Find! {}", DisplayPair(&target, &context.db)));
        for (label, formula) in context.clone().hypotheses().iter() {
            context.message(&format!("Trying {}", DisplayPair(formula, &context.db)));
            match self.check_match(context, &target, &*formula, |_subst| {
                Ok(ProofStep::hyp(*label, formula.clone()))
            }) {
                Ok(step) => {
                    context.exit(&format!("Matched hypothesis {}", DisplayPair(formula, &context.db)));
                    return Ok(step);
                },
                Err(e) => {
                    context.message(&format!("{:?}", e));
                },
            }
        }
        for (hyp, step) in context.clone().subgoals().iter() {
            context.message(&format!("Trying {}", DisplayPair(hyp, &context.db)));
            match self.check_match(context, &target, &*hyp, |_subst| {
                Ok(step.clone())
            }) {
                Ok(step) => {
                    context.exit(&format!("Matched subgoal {}", DisplayPair(hyp, &context.db)));
                    return Ok(step);
                },
                Err(e) => {
                    context.message(&format!("{:?}", e));
                },
            }
        }
        context.exit("Find: No match found");
        Err(TacticsError::NoMatchFound)
    }
}

impl FindHyp {
    fn check_match<F>(&self, context: &Context, target: &Formula, formula: &Formula, make_proof_step: F) -> TacticsResult
        where F: Fn(&Box<Substitutions>) -> TacticsResult {
        let mut subst = Substitutions::new();
        formula.unify(&target, & mut subst)?;
        let step1 = make_proof_step(&Box::new(subst.clone()))?;
        let mut context2 = context.with_variables(&subst);
        context2.add_subgoal(step1.result().clone(), step1);
        Ok(self.tactics.execute(&mut context2)?)
    }
}
