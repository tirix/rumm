use crate::lang::DisplayPair;
use crate::lang::SubstitutionListExpression;
use metamath_rs::formula::Substitutions;
use crate::lang::TacticsExpression;
use crate::context::Context;
use crate::error::Result;
use crate::lang::ProofStep;
use crate::lang::{Db, Display};
use crate::lang::StatementExpression;
use crate::parser::{Parse, Parser, OptionalTactics};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;


/// A tactics which applies a given theorem to prove the goal.
///
pub struct Apply {
    theorem: StatementExpression,
    subtactics: Vec<TacticsExpression>,
    substitutions: SubstitutionListExpression,
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
        let theorem = StatementExpression::parse(parser)?;
        let mut subtactics = Vec::new();
        let mut substitutions = SubstitutionListExpression::default();
        loop {
            match parser.parse_optional_tactics()? {
                OptionalTactics::Some(t) => subtactics.push(t),
                OptionalTactics::None => break,
                OptionalTactics::With => {
                    substitutions = SubstitutionListExpression::parse(parser)?;
                    break;
                }
            }
        }
        
        Ok(Apply {
            theorem,
            subtactics,
            substitutions,
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
        context.enter(&format!("Apply {}", DisplayPair(&self.theorem, &context.db)));
        // context.db.debug_formula(context.goal());
        // println!("  vars:{}", DisplayPair(context.variables(), &context.db));
        let mut my_subst = Substitutions::default();
        for (l,f) in self.substitutions.evaluate(context)?.iter() {
            context.message(format!("Subst: {} {}", DisplayPair(l, &context.db), DisplayPair(f, &context.db)).as_str());
            my_subst.insert(*l, f.substitute(context.variables()));
        }

        let theorem = self.theorem.evaluate(context)?;
        context.message(&format!(" Attempting apply {}", DisplayPair(&theorem, &context.db)));
        if let Some((theorem_formula, hyps)) = context.get_theorem_formulas(theorem) {
            let mut subst = Substitutions::new();
            if let Err(e) = context.goal().unify(&theorem_formula, &mut subst) {
                context.exit("Apply statement doesn't match");
                return Err(e.into());
            }
            subst.extend(&my_subst);
            // context.message(&format!("  subst:{}", DisplayPair(&subst, &context.db)));
            if hyps.len() == self.subtactics.len() {
                let mut substeps = vec![];
                // TODO check count!
                for ((_hyp_label, hyp_formula), tactics) in hyps.iter().zip(&self.subtactics) {
                    let sub_goal = hyp_formula.substitute(&subst);
                    let mut sub_context = context.with_goal(sub_goal);
                    substeps.push(tactics.execute(&mut sub_context)?);
                }
                context.exit("Apply Unification success");
                Ok(ProofStep::apply(
                    theorem,
                    substeps.into_boxed_slice(),
                    context.goal().clone(),
                    Box::new(subst),
                ))
            } else {
                context.exit("Apply Hyps don't match");
                Err(TacticsError::WrongHypCount(self.subtactics.len(), hyps.len()))
            }
        } else {
            context.exit("Unknown theorem label");
            Err(TacticsError::UnknownLabel(theorem))
        }
    }
}
