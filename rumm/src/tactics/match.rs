use crate::context::Context;
use crate::error::Result;
use crate::lang::DisplayPair;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;
use metamath_knife::Formula;

/// A tactics which tries a list of tactics until one of them produces a proof.
///
pub struct Match {
    formula: Formula,
    matches: Vec<(Formula, Box<dyn Tactics>)>,
}

impl Display for Match {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Match \n")?;
        self.formula.format(fmt, db)?;
        for (f, t) in &self.matches {
            f.format(fmt, db)?;
            t.format(fmt, db)?;
        }
        fmt.write_str("}\n")
    }
}

impl Parse for Match {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let formula = parser.parse_formula()?;
        let mut matches = Vec::new();
        while let Some(f) = parser.parse_optional_formula()? {
            matches.push((f, parser.parse_tactics()?));
        }
        Ok(Match { formula, matches })
    }
}

impl Tactics for Match {
    fn get_name(&self) -> String {
        "try".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which tries a list of tactics until one of them produces a proof.".to_string()
    }

    fn execute(&self, context: &mut Context) -> TacticsResult {
        let model = self.formula.substitute(context.variables());
        for m in self.matches.iter() {
            if let Some(subst) = model.unify(&m.0) {
                println!(
                    "Matched {} with {}",
                    DisplayPair(&self.formula, &context.db),
                    DisplayPair(&m.0, &context.db)
                );
                let mut sub_context = context.with_variables(&subst);
                if let Ok(step) = m.1.execute(&mut sub_context) {
                    return Ok(step);
                }
            }
        }
        println!("Match failed");
        Err(TacticsError::Error)
    }
}
