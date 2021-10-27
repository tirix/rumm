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

    fn execute(&self, context: &mut Context) -> TacticsResult {
        let target = self.formula.evaluate(context)?.substitute(context.variables());
        println!("-- Find -- {}", DisplayPair(&target, &context.db));
        for (label, formula) in context.clone().hypotheses().iter() {
            println!("Trying {}", DisplayPair(formula, &context.db));
            if let Ok(step) = self.check_match(context, &target, &*formula, |_subst| {
                Ok(ProofStep::hyp(*label, formula.clone()))
            }) {
                return Ok(step);
            }
        }
        // TODO also go through subgoals
        for (label, formula, hyps) in context.clone().statements() {
            if let Ok(step) = self.check_match(context, &target, &formula, |subst| {
                println!("Found match with {}", DisplayPair(&label, &context.db));
                println!("  subst:{}", DisplayPair(subst, &context.db));
                let mut substeps = vec![];
                let mut failed = false;
                for (_hyp_label, hyp_formula) in hyps.iter() {
                    let sub_goal = hyp_formula.substitute(&subst);
                    let mut sub_context = context.with_goal(sub_goal).with_variables(&subst);
                    if let Ok(sub_step) = self.tactics1.execute(&mut sub_context) {
                        substeps.push(sub_step);
                    } else {
                        failed = true;
                    }
                }
                if failed { return Err(TacticsError::Error); };
                println!("Unification success");
                let subgoal = formula.substitute(&subst);
                println!("  subgoal = {}", DisplayPair(&subgoal, &context.db));
                let subgoal_subst = subgoal.unify(&formula).ok_or_else(|| { TacticsError::Error })?;
                Ok(ProofStep::apply(
                    label,
                    substeps.into_boxed_slice(),
                    subgoal.clone(),
                    subgoal_subst.clone(),
                ))
            }) {
                return Ok(step);
            }
        }
        println!("No match found.");
        Err(TacticsError::Error)
    }
}

impl Find {
    fn check_match<F>(&self, context: &Context, target: &Formula, formula: &Formula, make_proof_step: F) -> TacticsResult
        where F: Fn(&Box<Substitutions>) -> TacticsResult {
        if let Some(subst) = formula.unify(&target) {
            let step1 = make_proof_step(&subst)?;
            let mut context2 = context.with_variables(&subst);
            context2.add_subgoal(step1.result().clone(), step1);
            Ok(self.tactics2.execute(&mut context2)?)
        } else {
            Err(TacticsError::Error)
        }
    }
}
