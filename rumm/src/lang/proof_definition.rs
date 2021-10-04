use crate::context::Context;
use crate::error::Result;
use crate::lang::TacticsDict;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::Tactics;
use crate::tactics::TacticsError;
use crate::tactics::TacticsResult;
use core::fmt::Formatter;
use metamath_knife::formula::Substitutions;
use metamath_knife::proof::ProofTreeArray;
use metamath_knife::Formula;
use metamath_knife::Label;

/// One step in a proof
pub struct ProofStep {
    apply: Label,
    apply_on: Box<[ProofStep]>,
    result: Formula,
    substitutions: Box<Substitutions>,
}

impl ProofStep {
    pub fn new(
        apply: Label,
        apply_on: Box<[ProofStep]>,
        result: Formula,
        substitutions: Box<Substitutions>,
    ) -> Self {
        ProofStep {
            apply,
            apply_on,
            result,
            substitutions,
        }
    }

    fn add_to_proof_tree_array(
        &self,
        stack_buffer: &mut Vec<u8>,
        arr: &mut ProofTreeArray,
        db: Db,
    ) -> Option<usize> {
        let hyps = self
            .apply_on
            .iter()
            .map(|step| step.add_to_proof_tree_array(stack_buffer, arr, db.clone()))
            .flatten()
            .collect();
        db.build_proof(
            self.apply,
            self.result.clone(),
            hyps,
            &self.substitutions,
            stack_buffer,
            arr,
        )
    }

    pub fn as_proof_tree_array(&self, db: Db) -> ProofTreeArray {
        let mut arr = ProofTreeArray::default();
        let mut stack_buffer = vec![];
        arr.qed = self
            .add_to_proof_tree_array(&mut stack_buffer, &mut arr, db)
            .unwrap();
        arr
    }
}

pub struct ProofDefinition {
    theorem: Label,
    tactics: Box<dyn Tactics>,
}

impl Display for ProofDefinition {
    fn format(&self, fmt: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str("proof ")?;
        self.theorem.format(fmt, db)?;
        self.tactics.format(fmt, db)
    }
}

impl Parse for ProofDefinition {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let theorem = parser.parse_theorem_label()?;
        let tactics = parser.parse_tactics()?;
        Ok(ProofDefinition { theorem, tactics })
    }
}

impl ProofDefinition {
    pub fn prove(&self, db: Db, tactics_definitions: TacticsDict) -> TacticsResult {
        if let Some((theorem_formula, hypotheses)) = db.get_theorem_formulas(self.theorem) {
            let mut context =
                Context::new(db.clone(), theorem_formula, hypotheses, tactics_definitions);
            println!("Proof for {:?}:", self.theorem.to_string(&db));
            self.tactics.execute(&mut context)
        } else {
            println!("Unknown theorem!");
            Err(TacticsError::Error)
        }
    }

    pub fn theorem(&self) -> Label {
        self.theorem
    }
}
