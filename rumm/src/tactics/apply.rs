use crate::context::Context;
use crate::error::Result;
use crate::lang::ProofStep;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;
use metamath_knife::Label;

/// A tactics which applies a given theorem to prove the goal.
///
pub struct Apply {
    theorem: Label,
    subtactics: Vec<Box<dyn Tactics>>,
}

impl Display for Apply {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Apply \n")?;
        self.theorem.format(fmt, db)?;
        for t in &self.subtactics {
            t.format(fmt, db)?;
        }
        fmt.write_str("}\n")
    }
}

impl Parse for Apply {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let theorem = parser.parse_theorem_label()?;
        let mut subtactics = Vec::new();
        while let Some(t) = parser.parse_optional_tactics()? {
            subtactics.push(t);
        }
        // TODO check count!
        Ok(Apply {
            theorem,
            subtactics,
        })
    }
}

impl Tactics for Apply {
    fn get_name(&self) -> String {
        "apply".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which applies a given theorem to prove the goal.".to_string()
    }

    fn execute(&self, context: &mut Context) -> TacticsResult {
        if let Some((theorem_formula, hyps)) = context.get_theorem_formulas(self.theorem) {
            if let Some(subst) = context.goal().unify(&theorem_formula) {
                println!("Attempting apply");
                if hyps.len() == self.subtactics.len() {
                    let mut substeps = vec![];
                    for ((_hyp_label, hyp_formula), tactics) in hyps.iter().zip(&self.subtactics) {
                        let sub_goal = hyp_formula.substitute(&subst);
                        let mut sub_context = context.with_goal(sub_goal);
                        substeps.push(tactics.execute(&mut sub_context)?);
                    }
                    println!("Unification success");
                    Ok(ProofStep::new(
                        self.theorem,
                        substeps.into_boxed_slice(),
                        context.goal().clone(),
                        subst,
                    ))
                } else {
                    println!("Hyps don't match");
                    Err(TacticsError::Error)
                }
            } else {
                println!("Unification failure");
                Err(TacticsError::Error)
            }
        } else {
            println!("Unknown theorem label");
            Err(TacticsError::Error)
        }
    }
}
