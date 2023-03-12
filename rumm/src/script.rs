use colored::Colorize;

use crate::error::Result;
use crate::lang::{Db, Display};
use crate::lang::{ProofDefinition, TacticsDefinition, TacticsDict};
use crate::parser::{Parse, Parser};
use crate::trace::Trace;
use core::fmt::{Debug, Formatter};

pub struct Script {
    pub(crate) db: Db,
    pub(crate) tactics_definitions: TacticsDict,
    pub(crate) proof_definitions: Vec<ProofDefinition>,
}

impl Debug for Script {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("Script {")?;
        fmt.write_str("tactics_definitions: [")?;
        self.tactics_definitions.format(fmt, &self.db)?;
        fmt.write_str("]\n")?;
        fmt.write_str("proofs_definitions: [")?;
        for p in &self.proof_definitions {
            p.format(fmt, &self.db)?;
        }
        fmt.write_str("]\n")?;
        fmt.write_str("}\n")
    }
}

impl Parse for Script {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.parse_script()
    }
}

impl Script {
    pub fn new(
        db: Db,
        tactics_list: Vec<TacticsDefinition>,
        proof_definitions: Vec<ProofDefinition>,
    ) -> Self {
        Script {
            db,
            tactics_definitions: TacticsDict::from(tactics_list),
            proof_definitions,
        }
    }

    pub fn from_str(filename: String, str: &str) -> Result<Self> {
        Script::parse(&mut Parser::from_str(filename, str))
    }

    pub fn execute(&mut self) -> Result {
        for proof_def in &self.proof_definitions {
            let mut trace = Trace::new();
            print!("Proving {} ... ", &proof_def.theorem().to_string(&self.db));
            match proof_def.prove(self.db.clone(), self.tactics_definitions.clone(), &mut trace) {
                Ok(_step) => {
                    println!("{}", "ok".green());
                    // let mut arr = step.as_proof_tree_array(self.db.clone());
                    // arr.calc_indent();
                    // self.db.export_mmp(proof_def.theorem(), &arr, &mut std::io::stdout());
                }
                Err(_) => {
                    println!("{}", "failed".red());
                    //trace.dump();
                    trace.export_js_tree(&proof_def.theorem().to_string(&self.db));
                }
            }
        }
        Ok(())
    }
}
