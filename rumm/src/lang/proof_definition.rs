use crate::lang::TacticsExpression;
use crate::context::Context;
use crate::error::Result;
use crate::lang::TacticsDict;
use crate::lang::{Db, Display};
use crate::parser::{Parse, Parser};
use crate::tactics::TacticsError;
use crate::trace::Trace;
use core::fmt::Formatter;
use metamath_knife::formula::Substitutions;
use metamath_knife::proof::ProofTreeArray;
use metamath_knife::Formula;
use metamath_knife::Label;

#[derive(Clone)]
/// One step in a proof
pub enum ProofStep {
	Apply {
	    apply: Label,
	    apply_on: Box<[ProofStep]>,
	    result: Formula,
	    substitutions: Box<Substitutions>,
	},
	Hyp {
		label: Label,
		result: Formula,
	},
}

impl ProofStep {
    pub fn apply(
        apply: Label,
        apply_on: Box<[ProofStep]>,
        result: Formula,
        substitutions: Box<Substitutions>,
    ) -> Self {
        ProofStep::Apply {
            apply,
            apply_on,
            result,
            substitutions,
        }
    }

    pub fn hyp(label: Label, result: Formula) -> Self {
    	ProofStep::Hyp { label, result }
    }

    pub fn result(&self) -> &Formula {
        match self {
            ProofStep::Apply { result: r, .. } => r,
            ProofStep::Hyp { result: r, .. } => r,
        }
    }

    fn add_to_proof_tree_array(
        &self,
        stack_buffer: &mut Vec<u8>,
        arr: &mut ProofTreeArray,
        db: Db,
    ) -> Option<usize> {
    	match self {
    		ProofStep::Apply { apply, apply_on, result, substitutions } => {
		        let hyps = apply_on
		            .iter()
		            .map(|step| step.add_to_proof_tree_array(stack_buffer, arr, db.clone()))
		            .flatten()
		            .collect();
		        db.build_proof_step(
		            *apply,
		            result.clone(),
		            hyps,
		            &substitutions,
		            stack_buffer,
		            arr,
		        )
    		},
    		ProofStep::Hyp { label, result } => {
                db.build_proof_hyp(
                    *label,
                    result.clone(),
                    stack_buffer,
                    arr,
                )
    		},
    	}
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
    tactics: TacticsExpression,
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
    pub fn prove(&self, db: Db, tactics_definitions: TacticsDict, trace: &mut Trace) -> std::result::Result<ProofStep, TacticsError> {
        if let Some((theorem_formula, essential_hypotheses)) = db.get_theorem_formulas(self.theorem) {
            let mut context = Context::new(db.clone(), theorem_formula, essential_hypotheses, tactics_definitions);
            self.tactics.execute(trace, &mut context)
        } else {
            println!("Unknown theorem {:?}!", self.theorem);
            Err(TacticsError::UnknownLabel(self.theorem))
        }
    }

    pub fn theorem(&self) -> Label {
        self.theorem
    }
}
