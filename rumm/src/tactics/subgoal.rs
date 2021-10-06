use crate::context::Context;
use crate::error::Result;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;
use metamath_knife::formula::Formula;

/// A tactics which first solves subgoal,
/// then provides that subgoal as part of the proven statements for the subsequent part of the proof.
///
pub struct Subgoal {
    tactics1: Box<dyn Tactics>,
    subgoal: Formula,
    tactics2: Box<dyn Tactics>,
}

impl Parse for Subgoal {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let tactics1 = parser.parse_tactics()?;
        let subgoal = parser.parse_formula()?;
        let tactics2 = parser.parse_tactics()?;
        parser.parse_curly_bracket_close()?;
        Ok(Subgoal {
            tactics1,
            subgoal,
            tactics2,
        })
    }
}

impl Display for Subgoal {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Subgoal \n")?;
        self.tactics1.format(fmt, db)?;
        self.subgoal.format(fmt, db)?;
        self.tactics2.format(fmt, db)?;
        fmt.write_str("}\n")
    }
}

impl Tactics for Subgoal {
    fn get_name(&self) -> String {
        "subgoal".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which allows to insert a subgoal, prove it or assume it, and then move forward with the rest of the proof.".to_string()
    }

    fn execute(&self, mut context: &mut Context) -> TacticsResult {
        let mut context1 = context.with_goal(self.subgoal.clone());
        if let Ok(step1) = self.tactics1.execute(&mut context1) {
            context.add_subgoal(self.subgoal.clone(), step1);
            self.tactics2.execute(&mut context)
        } else {
            Err(TacticsError::Error)
        }
    }
}
