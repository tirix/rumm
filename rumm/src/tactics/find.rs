use metamath_rs::as_str;
use metamath_rs::formula::Substitutions;
use metamath_rs::Formula;
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

/// Generic trait for finding a theorem among all database statements to prove the goal.
trait FilterFind {
    fn filter(&self, is_axiom: bool, label: &[u8]) -> bool;
}

impl dyn FilterFind {
    fn find(&self, context: &mut Context, tactics1: &TacticsExpression, formula: &FormulaExpression, tactics2: &TacticsExpression) -> TacticsResult {
        let target = formula.evaluate(context)?.substitute(context.variables());
        context.enter(&format!("Find {}", DisplayPair(&target, &context.db)));
        for (label, formula) in context.clone().hypotheses().iter() {
            context.message(&format!("Trying {}", DisplayPair(formula, &context.db)));
            if let Ok(step) = self.check_match(context, &target, &*formula, tactics2, |_subst| {
                Ok(ProofStep::hyp(*label, formula.clone()))
            }) {
                context.exit(&format!("Matched hypothesis {}", DisplayPair(formula, &context.db)));
                return Ok(step);
            }
        }
        for (hyp, step) in context.clone().subgoals().iter() {
            context.message(&format!("Trying {}", DisplayPair(hyp, &context.db)));
            if let Ok(step) = self.check_match(context, &target, &*hyp, tactics2, |_subst| {
                Ok(step.clone())
            }) {
                context.exit(&format!("Matched subgoal {}", DisplayPair(hyp, &context.db)));
                return Ok(step);
            }
        }
        for (label, formula, hyps) in context.clone().statements(|is_axiom, label| self.filter(is_axiom, label)) {
            if let Ok(step) = self.check_match(context, &target, &formula, &tactics2,|subst| {
                context.message(&format!("Found match with {}", DisplayPair(&label, &context.db)));
                // context.message(&format!("  subst:{}", DisplayPair(subst, &context.db)));
                let mut substeps = vec![];
                let mut failed = false;
                for (_hyp_label, hyp_formula) in hyps.iter() {
                    let sub_goal = hyp_formula.substitute(&subst);
                    let mut sub_context = context.with_goal(sub_goal).with_variables(&subst);
                    if let Ok(sub_step) = tactics1.execute(&mut sub_context) {
                        substeps.push(sub_step);
                    } else {
                        failed = true;
                    }
                }
                if failed { return Err(TacticsError::NoMatchFound); };
                context.message("Unification success");
                let subgoal = formula.substitute(&subst);
                context.message(&format!("  subgoal = {}", DisplayPair(&subgoal, &context.db)));
                let mut subgoal_subst = Substitutions::new();
                subgoal.unify(&formula, &mut subgoal_subst)?;
                Ok(ProofStep::apply(
                    label,
                    substeps.into_boxed_slice(),
                    subgoal.clone(),
                    Box::new(subgoal_subst.clone()),
                ))
            }) {
                context.exit("Find Successful!");
                return Ok(step);
            }
        }
        context.exit("Find: No match found");
        Err(TacticsError::NoMatchFound)
    }

    fn check_match<F>(&self, context: &Context, target: &Formula, formula: &Formula, tactics2: &TacticsExpression, make_proof_step: F) -> TacticsResult
        where F: Fn(&Box<Substitutions>) -> TacticsResult {
        let mut subst = Substitutions::new();
        target.unify(&formula, & mut subst)?;
        let step1 = make_proof_step(&Box::new(subst.clone()))?;
        let mut context2 = context.with_variables(&subst);
        context2.add_subgoal(step1.result().clone(), step1);
        Ok(tactics2.execute(&mut context2)?)
    }
}

/// A tactics which finds a theorem in the whole statement list to prove the goal.
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

impl FilterFind for Find {
    fn filter(&self, _is_axiom: bool, _label: &[u8]) -> bool {
        true // No filtering
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
        <dyn FilterFind>::find(self, context, &self.tactics1, &self.formula, &self.tactics2)
    }
}

/// A tactics which finds a definition to prove the goal.
///
pub struct FindDf {
    tactics1: TacticsExpression,
    formula: FormulaExpression,
    tactics2: TacticsExpression,
}

impl Display for FindDf {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ FindDf \n")?;
        self.tactics1.format(fmt, db)?;
        self.formula.format(fmt, db)?;
        self.tactics2.format(fmt, db)?;
        fmt.write_str("}\n")
    }
}

impl Parse for FindDf {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let tactics1 = parser.parse_tactics()?;
        let formula = FormulaExpression::parse(parser)?;
        let tactics2 = parser.parse_tactics()?;
        parser.parse_curly_bracket_close()?;
        Ok(FindDf {
            tactics1,
            formula,
            tactics2,
        })
    }
}

impl FilterFind for FindDf {
    fn filter(&self, is_axiom: bool, label: &[u8]) -> bool {
        is_axiom && as_str(label).starts_with("df-")
    }
}

impl Tactics for FindDf {
    fn get_name(&self) -> String {
        "finddf".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which searches for a definition matching the given formula.".to_string()
    }
    
    fn execute(&self, context: &mut Context) -> TacticsResult {
        <dyn FilterFind>::find(self, context, &self.tactics1, &self.formula, &self.tactics2)
    }
}
