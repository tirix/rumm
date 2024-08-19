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


/// A tactics which applies a given theorem to prove the goal.
///
pub struct Find {
    tactics1: TacticsExpression,
    formula: FormulaExpression,
    tactics2: TacticsExpression,
}

impl Display for Find {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Find \n")?;
        self.tactics1.format(fmt, db)?;
        self.formula.format(fmt, db)?;
        self.tactics2.format(fmt, db)?;
        fmt.write_str("}\n")
    }
}

impl Parse for Find {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let tactics1 = parser.parse_tactics()?;
        let formula = FormulaExpression::parse(parser)?;
        let tactics2 = parser.parse_tactics()?;
        parser.parse_curly_bracket_close()?;
        Ok(Find {
            tactics1,
            formula,
            tactics2,
        })
    }
}

impl Tactics for Find {
    fn get_name(&self) -> String {
        "find".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which searches for a theorem matching the given formula.".to_string()
    }

    fn execute_intern(&self, trace: &mut Trace, context: &mut Context) -> TacticsResult {
        let target = self.formula.evaluate(context)?.substitute(context.variables());
        for (label, formula) in context.clone().hypotheses().iter() {
            trace.message(&format!("Trying {}", formula.to_string(&context.db)));
            if let Ok(step) = self.check_match(trace, context, &target, &*formula, |_subst, _trace| {
                Ok(ProofStep::hyp(*label, formula.clone()))
            }) {
                return Ok(step);
            }
        }
        for (hyp, step) in context.clone().subgoals().iter() {
            trace.message(&format!("Trying {}", hyp.to_string(&context.db)));
            if let Ok(step) = self.check_match(trace, context, &target, &*hyp, |_subst, _trace| {
                Ok(step.clone())
            }) {
                return Ok(step);
            }
        }
        for (label, formula, hyps) in context.clone().statements() {
            if let Ok(step) = self.check_match(trace, context, &target, &formula, |subst, trace: &mut Trace| {
                trace.message(&format!("Found match with {}", &label.to_string(&context.db)));
                // trace.message(&format!("  subst:{}", subst.to_string(&context.db)));
                let mut substeps = vec![];
                let mut failed = false;
                for (_hyp_label, hyp_formula) in hyps.iter() {
                    let sub_goal = hyp_formula.substitute(&subst);
                    let mut sub_context = context.with_goal(sub_goal).with_variables(&subst);
                    if let Ok(sub_step) = self.tactics1.execute(trace, &mut sub_context) {
                        substeps.push(sub_step);
                    } else {
                        failed = true;
                    }
                }
                if failed { return Err(TacticsError::NoMatchFound); };
                trace.message("Unification success");
                let subgoal = formula.substitute(&subst);
                trace.message(&format!("  subgoal = {}", &subgoal.to_string(&context.db)));
                let mut subgoal_subst = Substitutions::new();
                subgoal.unify(&formula, &mut subgoal_subst)?;
                Ok(ProofStep::apply(
                    label,
                    substeps.into_boxed_slice(),
                    subgoal.clone(),
                    Box::new(subgoal_subst.clone()),
                ))
            }) {
                return Ok(step);
            }
        }
        Err(TacticsError::NoMatchFound)
    }
}

impl Find {
    fn check_match<F>(&self, trace: &mut Trace, context: &Context, target: &Formula, formula: &Formula, make_proof_step: F) -> TacticsResult
        where F: Fn(&Box<Substitutions>, &mut Trace) -> TacticsResult {
        let mut subst = Substitutions::new();
        formula.unify(&target, & mut subst)?;
        let step1 = make_proof_step(&Box::new(subst.clone()), trace)?;
        let mut context2 = context.with_variables(&subst);
        context2.add_subgoal(step1.result().clone(), step1);
        Ok(self.tactics2.execute(trace, &mut context2)?)
    }
}
