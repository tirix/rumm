use crate::context::Context;
use crate::error::Result;
use crate::lang::ProofStep;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;

/// A tactics which matches the goal with one of the hypothesis, or a zero-hypothesis theorem or axiom
///
#[derive(Debug)]
pub struct Hypothesis;

impl Parse for Hypothesis {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.parse_curly_bracket_close()?;
        Ok(Hypothesis {})
    }
}

impl Display for Hypothesis {
    fn format(&self, fmt: &mut Formatter, _db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("!")
    }
}

impl Tactics for Hypothesis {
    fn get_name(&self) -> String {
        "!".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which matches the goal with one of the hypothesis.".to_string()
    }

    fn execute(&self, context: &mut Context) -> TacticsResult {
        context.enter("!");
        for (label, hyp) in context.hypotheses().iter() {
            if context.goal().eq(hyp) {
                context.exit("Matched hypothesis!");
                return Ok(ProofStep::hyp(
                    *label,
                    context.goal().clone(),
                ));
            }
        }
        for (hyp, step) in context.subgoals().iter() {
            if context.goal().eq(hyp) {
                context.exit("Matched subgoal!");
                return Ok(step.clone());
            }
        }
        context.exit("Hypothesis failed");
        Err(TacticsError::NoMatchFound)
    }
}
