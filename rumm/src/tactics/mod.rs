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
use metamath_knife::Label;
use metamath_knife::formula::TypeCode;
use metamath_knife::formula::UnificationError;
pub use r#match::Match;
pub use r#try::Try;
pub use skipped::Skipped;
pub use subgoal::Subgoal;
pub use find_hyp::FindHyp;
pub use find::Find;
pub use use_script_tactics::UseScriptTactics;

use crate::context::Context;
use crate::lang::Display;
use crate::lang::ProofStep;
use crate::parser::Parse;
use crate::trace::Trace;

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
    fn execute_intern(&self, trace: &mut Trace, context: &mut Context) -> TacticsResult;
    fn execute (&self, trace: &mut Trace, context: &mut Context) -> TacticsResult where Self: Sized {
        let mut trace1 = trace.enter(context, &self.to_string(&context.db));
        let result = self.execute_intern(&mut trace1, context);
        match result {
            TacticsResult::Ok(_) => { trace.exit(trace1, &format!("{} : success", &self.to_string(&context.db))); }
            TacticsResult::Err(_) => { trace.exit(trace1, &format!("{} : failure", &self.to_string(&context.db))); } // "{e:?}"
        }
        result
    }

    /// Return a Arc to the tactics.
    fn into_arc(self) -> Arc<dyn Tactics>
    where
        Self: 'static + Sized,
    {
        Arc::new(self)
    }
}
