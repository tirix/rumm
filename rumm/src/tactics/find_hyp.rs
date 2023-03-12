use metamath_knife::formula::Substitutions;
use metamath_knife::Formula;
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
use crate::trace::Trace;
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

    fn execute_intern(&self, trace: &mut Trace, context: &mut Context) -> TacticsResult {
        let target = self.formula.evaluate(context)?.substitute(context.variables());
        trace.message(&format!("Find! {}", &target.to_string(&context.db)));
        for (label, formula) in context.clone().hypotheses().iter() {
            trace.message(&format!("Trying {}", formula.to_string(&context.db)));
            match self.check_match(trace, context, &target, &*formula, |_subst| {
                Ok(ProofStep::hyp(*label, formula.clone()))
            }) {
                Ok(step) => {
                    return Ok(step);
                },
                Err(e) => {
                    trace.message(&format!("{:?}", e));
                },
            }
        }
        for (hyp, step) in context.clone().subgoals().iter() {
            trace.message(&format!("Trying {}", hyp.to_string(&context.db)));
            match self.check_match(trace, context, &target, &*hyp, |_subst| {
                Ok(step.clone())
            }) {
                Ok(step) => {
                    return Ok(step);
                },
                Err(e) => {
                    trace.message(&format!("{:?}", e));
                },
            }
        }
        Err(TacticsError::NoMatchFound)
    }
}

impl FindHyp {
    fn check_match<F>(&self, trace: &mut Trace, context: &Context, target: &Formula, formula: &Formula, make_proof_step: F) -> TacticsResult
        where F: Fn(&Box<Substitutions>) -> TacticsResult {
        let mut subst = Substitutions::new();
        formula.unify(&target, & mut subst)?;
        let step1 = make_proof_step(&Box::new(subst.clone()))?;
        let mut context2 = context.with_variables(&subst);
        context2.add_subgoal(step1.result().clone(), step1);
        Ok(self.tactics.execute(trace, &mut context2)?)
    }
}
