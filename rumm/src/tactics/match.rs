use crate::lang::TacticsExpression;
use crate::trace::Trace;
use metamath_knife::Formula;
use metamath_knife::formula::Substitutions;
use crate::lang::FormulaExpression;
use crate::context::Context;
use crate::error::Result;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;

/// A tactics which tries a list of tactics until one of them produces a proof.
///
pub struct Match {
    target: FormulaExpression,
    matches: Vec<(Formula, TacticsExpression)>,
}

impl Display for Match {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("{ Match\n")?;
        self.target.format(fmt, db)?;
        for (f, t) in &self.matches {
            f.format(fmt, db)?;
            t.format(fmt, db)?;
        }
        fmt.write_str("}\n")
    }
}

impl Parse for Match {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let target = parser.parse_formula_expression()?;
        let mut matches = Vec::new();
        while let Some(f) = parser.parse_optional_formula()? {
            matches.push((f, parser.parse_tactics()?));
        }
        Ok(Match { target, matches })
    }
}

impl Tactics for Match {
    fn get_name(&self) -> String {
        "match".to_string()
    }

    fn get_desc(&self) -> String {
        "A tactics which tries matching against a list of templates until one of them produces a proof.".to_string()
    }

    fn execute_intern(&self, trace: &mut Trace, context: &mut Context) -> TacticsResult {
        let model = self.target.evaluate(&context)?.substitute(context.variables());
        // match &self.target {
        //     FormulaExpression::Formula(formula) => formula.substitute(context.variables()),
        //     FormulaExpression::Goal => context.goal().clone(),
        //     FormulaExpression::Statement(label) => context.get_theorem_formulas(label.evaluate(context)?)
        //         .ok_or(TacticsError::Error)?.0, // unknown statement
            
        // };
        trace.message(&format!("Target {}", &model.to_string(&context.db)));
//        context.message(format!("{}", context.debug_formula(&model)));
        for m in self.matches.iter() {
            let m2 = m.0.substitute(context.variables());
            trace.message(&format!("Trying {}", &m2.to_string(&context.db)));
//            context.message(format!("  {}", context.debug_formula(&m2))));
            let mut subst = Substitutions::new();
            if let std::result::Result::Ok(_) = model.unify(&m2, &mut subst) {
                trace.message(&format!(
                    "Matched {} with {}",
                    &model.to_string(&context.db),
                    &m2.to_string(&context.db)
                ));
                let mut sub_context = context.with_variables(&subst);
                match m.1.execute(trace, &mut sub_context) {
                    Ok(step) => {
                        return Ok(step);
                    },
                    Err(e) => {
                        trace.message(format!("{:?}", e).as_str());
                    },
                }
            }
        }
        Err(TacticsError::NoMatchFound)
    }
}
