use crate::lang::Db;
use crate::lang::DisplayPair;
use crate::lang::Hypotheses;
use crate::lang::TacticsDefinition;
use crate::lang::TacticsDict;
use core::fmt::{Debug, Formatter};
use metamath_knife::formula::Substitutions;
use metamath_knife::Formula;
use metamath_knife::Label;

#[derive(Clone)]
pub struct Context {
    pub(crate) db: Db,
    goal: Formula,
    hypotheses: Hypotheses,
    // subgoals TODO
    variables: Substitutions,
    tactics_definitions: TacticsDict,
}

impl Debug for Context {
    fn fmt(&self, _formatter: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl<'a> Context {
    pub fn new(
        db: Db,
        goal: Formula,
        hypotheses: Hypotheses,
        tactics_definitions: TacticsDict,
    ) -> Self {
        println!("Proving {}", DisplayPair(&goal, &db));
        let mut variables = Substitutions::default();
        db.set_goal(&mut variables, goal.clone());
        Context {
            db,
            goal,
            hypotheses,
            variables,
            tactics_definitions,
        }
    }

    pub fn get_tactics_definition(&self, name: String) -> Option<&TacticsDefinition> {
        self.tactics_definitions.get(name)
    }

    pub fn with_goal(&self, goal: Formula) -> Self {
        println!("Proving {}", DisplayPair(&goal, &self.db));
        let mut variables = self.variables.clone();
        self.db.set_goal(&mut variables, goal.clone());
        Self {
            db: self.db.clone(),
            goal,
            hypotheses: self.hypotheses.clone(),
            variables,
            tactics_definitions: self.tactics_definitions.clone(),
        }
    }

    pub fn with_hyp(&self, _formula: Formula) -> Self {
        Self {
            db: self.db.clone(),
            goal: self.goal.clone(),
            hypotheses: self.hypotheses.clone(),
            variables: self.variables.clone(),
            tactics_definitions: self.tactics_definitions.clone(),
        }
    }

    pub fn with_variables(&self, v: &Substitutions) -> Self {
        let mut variables = self.variables.clone();
        variables.extend(v);
        Self {
            db: self.db.clone(),
            goal: self.goal.clone(),
            hypotheses: self.hypotheses.clone(),
            variables,
            tactics_definitions: self.tactics_definitions.clone(),
        }
    }

    pub fn goal(&self) -> &Formula {
        &self.goal
    }
    pub fn variables(&self) -> &Substitutions {
        &self.variables
    }
    pub fn hypotheses(&self) -> &Hypotheses {
        &self.hypotheses
    }
    pub fn get_theorem_formulas(&self, label: Label) -> Option<(Formula, Hypotheses)> {
        self.db.get_theorem_formulas(label)
    }
}
