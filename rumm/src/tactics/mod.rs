//! Tactics for the Rumm proof assistant

mod apply;
mod hypothesis;
mod r#match;
mod skipped;
mod subgoal;
mod r#try;
mod find_hyp;
mod find;
mod use_script_tactics;

use std::sync::Arc;
pub use apply::Apply;
pub use hypothesis::Hypothesis;
use metamath_rs::Label;
use metamath_rs::formula::TypeCode;
use metamath_rs::formula::UnificationError;
pub use r#match::Match;
pub use r#try::Try;
pub use skipped::Skipped;
pub use subgoal::Subgoal;
pub use find_hyp::FindHyp;
pub use find::Find;
pub use find::FindDf;
pub use use_script_tactics::UseScriptTactics;

use crate::context::Context;
use crate::lang::Display;
use crate::lang::ProofStep;
use crate::parser::Parse;

pub type TacticsResult<T = ProofStep> = std::result::Result<T, TacticsError>;

#[derive(Debug)]
pub enum TacticsError {
    Skipped,
    UnknownTactics(String),
    UnificationFailed,
    NoMatchFound,
    WrongParameterCount(usize, usize),
    WrongHypCount(usize, usize),
    WrongTypecode(TypeCode, TypeCode, Label),
    UnknownLabel(Label),
    UnknownFormulaVariable(String),
    UnknownTacticsVariable(String),
    UnknownLabelVariable(String),
    UnknownSubstitutionVariable(String),
}

impl From<UnificationError> for TacticsError {
    fn from(_: UnificationError) -> Self {
        Self::UnificationFailed
    }
}

/// The trait implemented by all tactics.
///
pub trait Tactics: Parse + Display {
    fn get_name(&self) -> String;
    //fn arg_types(&self) ->
    fn get_desc(&self) -> String;
    fn execute(&self, context: &mut Context) -> TacticsResult;

    /// Return a Arc to the tactics.
    fn into_arc(self) -> Arc<dyn Tactics>
    where
        Self: 'static + Sized,
    {
        Arc::new(self)
    }
}
