//! Tactics for the Rumm proof assistant

mod apply;
mod hypothesis;
mod r#match;
mod skipped;
mod subgoal;
mod r#try;
mod use_script_tactics;

pub use apply::Apply;
pub use hypothesis::Hypothesis;
pub use r#match::Match;
pub use r#try::Try;
pub use skipped::Skipped;
pub use subgoal::Subgoal;
pub use use_script_tactics::UseScriptTactics;

use crate::context::Context;
use crate::lang::Display;
use crate::lang::ProofStep;
use crate::parser::Parse;

pub type TacticsResult = std::result::Result<ProofStep, TacticsError>;

// TODO Add relevant errors
pub enum TacticsError {
    Error,
}

/// The trait implemented by all tactics.
///
pub trait Tactics: Parse + Display {
    fn get_name(&self) -> String;
    //fn arg_types(&self) ->
    fn get_desc(&self) -> String;
    fn execute(&self, context: &mut Context) -> TacticsResult;

    /// Box the tactics.
    fn boxed(self) -> Box<dyn Tactics>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }
}
