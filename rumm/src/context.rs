use std::sync::Arc;
use crate::tactics::Tactics;
use crate::lang::ProofStep;
use crate::lang::Db;
use crate::lang::DisplayPair;
use crate::lang::Hypotheses;
use crate::lang::TacticsDefinition;
use crate::lang::TacticsDict;
use core::fmt::{Debug, Formatter};
use metamath_knife::formula::Substitutions;
use metamath_knife::Formula;
use metamath_knife::Label;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Context {
    pub(crate) db: Db,
    goal: Formula,
    hypotheses: Hypotheses,
    subgoals: Vec<(Formula, ProofStep)>,
    tactics_definitions: TacticsDict,
    variables: Substitutions,
    label_variables: HashMap<String, Label>,
    tactics_variables: HashMap<String, Arc<dyn Tactics>>,
    formula_variables: HashMap<String, Formula>,
    subst_variables: HashMap<String, Substitutions>,
    depth: usize,
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
        let subgoals = vec![];
        Context {
            db,
            goal,
            hypotheses,
            subgoals,
            tactics_definitions,
            variables: Substitutions::default(),
            label_variables: HashMap::default(),
            tactics_variables: HashMap::default(),
            formula_variables: HashMap::default(),
            subst_variables: HashMap::default(),
            depth: 0,
        }
    }

    pub fn message(&self, message: &str) {
        println!("{:indent$}{message}", "", indent = self.depth);
    }

    pub fn enter(&self, message: &str) {
        self.message(&format!("Proving {}", DisplayPair(&self.goal, &self.db)));
        self.message(&format!(">> {message}"));
    }

    pub fn exit(&self, message: &str) {
        self.message(&format!("<< {message}"));
    }

    pub fn get_tactics_definition(&self, name: String) -> Option<&TacticsDefinition> {
        self.tactics_definitions.get(name)
    }

    pub fn with_goal(&self, goal: Formula) -> Self {
        Self {
            db: self.db.clone(),
            goal,
            hypotheses: self.hypotheses.clone(),
            subgoals: self.subgoals.clone(),
            tactics_definitions: self.tactics_definitions.clone(),
            variables: self.variables.clone(),
            label_variables: self.label_variables.clone(),
            tactics_variables: self.tactics_variables.clone(),
            formula_variables: self.formula_variables.clone(),
            subst_variables: self.subst_variables.clone(),
            depth: self.depth + 1,
        }
    }

    pub fn with_variables(&self, v: &Substitutions) -> Self {
        let mut variables = self.variables.clone();
        variables.extend(v);
        Self {
            db: self.db.clone(),
            goal: self.goal.clone(),
            hypotheses: self.hypotheses.clone(),
            subgoals: self.subgoals.clone(),
            tactics_definitions: self.tactics_definitions.clone(),
            variables,
            label_variables: self.label_variables.clone(),
            tactics_variables: self.tactics_variables.clone(),
            formula_variables: self.formula_variables.clone(),
            subst_variables: self.subst_variables.clone(),
            depth: self.depth + 1,
        }
    }

    pub fn without_variables(&self) -> Self {
        Self {
            db: self.db.clone(),
            goal: self.goal.clone(),
            hypotheses: self.hypotheses.clone(),
            subgoals: self.subgoals.clone(),
            tactics_definitions: self.tactics_definitions.clone(),
            variables: Substitutions::default(),
            label_variables: self.label_variables.clone(),
            tactics_variables: self.tactics_variables.clone(),
            formula_variables: self.formula_variables.clone(),
            subst_variables: self.subst_variables.clone(),
            depth: self.depth + 1,
        }
    }

    pub fn add_subgoal(&mut self, formula: Formula, step: ProofStep) {
        self.subgoals.push((formula, step));
    }
    pub fn add_label_variable(&mut self, id: String, label: Label) {
        self.label_variables.insert(id, label);
    }
    pub fn add_tactics_variable(&mut self, id: String, tactics: Arc<dyn Tactics>) {
        self.tactics_variables.insert(id, tactics);
    }
    pub fn add_formula_variable(&mut self, id: String, formula: Formula) {
        self.formula_variables.insert(id, formula);
    }
    pub fn add_substitution_variable(&mut self, id: String, subst: Substitutions) {
        self.subst_variables.insert(id, subst);
    }

    pub fn get_label_variable(&self, id: String) -> Option<Label> {
        self.label_variables.get(&id).map(|l| *l)
    }
    pub fn get_tactics_variable(&self, id: String) -> Option<Arc<dyn Tactics>> {
        self.tactics_variables.get(&id).map(|t| t.clone())
    }
    pub fn get_formula_variable(&self, id: String) -> Option<Formula> {
        self.formula_variables.get(&id).map(|f| f.clone())
    }
    pub fn get_substitution_variable(&self, id: String) -> Option<&Substitutions> {
        self.subst_variables.get(&id)
    }

    pub fn get_variable_label(&self, f: Formula) -> Option<Label> {
        f.get_by_path(&[])
    }

    pub fn goal(&self) -> &Formula {
        &self.goal
    }
    pub fn subgoals(&self) -> &Vec<(Formula, ProofStep)> {
        &self.subgoals
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
    pub fn statements(&self, filter: impl Fn(bool, &[u8]) -> bool) -> impl Iterator<Item = (Label, Formula, Hypotheses)> + '_ {
        self.db.statements(filter)
    }
//    pub fn debug_formula(&self, f: &Formula) -> String {
//        self.db.debug_formula(f)
//    }
}
