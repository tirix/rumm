use crate::error::{Error, Result};
use crate::lang::Display;
use crate::parser::Token;
use colored::*;
use core::cell::RefCell;
use core::fmt::Formatter;
use metamath_knife::database::DbOptions;
use metamath_knife::export::export_mmp_proof_tree;
use metamath_knife::formula::Substitutions;
use metamath_knife::parser::{as_str, StatementType};
use metamath_knife::proof::ProofTreeArray;
use metamath_knife::scopeck::Hyp;
use metamath_knife::verify::ProofBuilder;
use metamath_knife::Database;
use metamath_knife::Formula;
use metamath_knife::Label;
use metamath_knife::Symbol;
use std::io::Write;
use std::sync::Arc;

pub type Hypotheses = Box<[(Label, Formula)]>;

#[derive(Clone)]
pub struct Db {
    intern: Arc<RefCell<Database>>,
}

impl Default for Db {
    fn default() -> Self {
        let options = DbOptions {
            incremental: true,
            autosplit: false,
            ..Default::default()
        };
        Db {
            intern: Arc::new(RefCell::new(Database::new(options))),
        }
    }
}

impl Db {
    pub fn parse(&mut self, filename: String) -> Result {
        print!("Loading \"{}\" ... ", filename);
        self.intern.borrow_mut().parse(filename.clone(), Vec::new());
        let err = self.intern.borrow_mut().parse_result().parse_diagnostics();
        if !err.is_empty() {
            return Err(Error::DBError(err));
        }
        println!("{}", "ok".green());
        print!("Parsing \"{}\" ... ", filename);
        let gerr = self.intern.borrow_mut().grammar_result().diagnostics();
        if !gerr.is_empty() {
            return Err(Error::DBError(gerr));
        }
        println!("{}", "ok".green());
        //let testx = self.get_theorem_label("testx".to_string()).unwrap();
        //let (_f, ..) = self.get_theorem_formulas(testx).unwrap();
        //println!("TESTX {}", f.debug(self.intern.borrow_mut().name_result()));
        Ok(())
    }

    pub fn get_symbol(&self, name: String) -> Result<Symbol> {
        Ok(self
            .intern
            .borrow_mut()
            .name_result()
            .lookup_symbol(name.as_bytes())
            .ok_or(Error::msg(format!("Unknown symbol {}", name)))?
            .atom
        )
    }

    pub fn get_theorem_label(&self, name: String) -> Result<Label> {
        Ok(self
            .intern
            .borrow_mut()
            .name_result()
            .lookup_label(name.as_bytes())
            .ok_or(Error::parse_error(
                "A known theorem label",
                Token::TheoremLabel(name),
            ))?.atom
        )
    }

    pub fn set_goal(&self, substitutions: &mut Substitutions, goal: Formula) {
        // TODO make this faster!
        let goal_label = self.get_theorem_label("wgoal".to_string()).unwrap();
        substitutions.insert(goal_label, goal);
    }

    fn get_theorem_formulas_internal(database: &mut Database, label: Label) -> Option<(Formula, Hypotheses)> {
        let nset = database.name_result().clone();
        let sset = database.parse_result().clone();
        let token = nset.atom_name(label);
        let lookup_label = nset.lookup_label(token)?;
        let sref = sset.statement(lookup_label.address);
        let formula = database
            .stmt_parse_result()
            .get_formula(&sref)
            .map(Formula::clone)?;
        let mut hypotheses = Vec::new();
        let scope = database.scope_result().clone();
        let frame = scope.get(token)?;
        for hyp in frame.hypotheses.iter() {
            if let Hyp::Essential(sa, _) = hyp {
                let sref = sset.statement(*sa);
                let label = nset
                    .lookup_label(sref.label())
                    .map_or(Label::default(), |l| l.atom);
                let formula = database
                    .stmt_parse_result()
                    .get_formula(&sref)
                    .map(Formula::clone)?;
                hypotheses.push((label, formula));
            }
        }
        Some((formula, hypotheses.into_boxed_slice()))
    }

    /// The first item in the tuple shall be the desired formula, the second, the list of essential hypotheses
    pub fn get_theorem_formulas(&self, label: Label) -> Option<(Formula, Hypotheses)> {
        let mut database = self.intern.borrow_mut();
        Self::get_theorem_formulas_internal(&mut database, label)
    }

    pub fn parse_formula(&self, symbols: Vec<Symbol>) -> Result<Formula> {
        let mut database = self.intern.borrow_mut();
        let grammar = database.grammar_result().clone();
        let nset = database.name_result();
        grammar
            .parse_formula(&mut symbols.into_iter(), &grammar.typecodes(), nset)
            .map_err(|diag| Error::msg(format!("{:?}", diag)))
    }

    // pub fn debug_formula(&self, f: &Formula) -> String {
    //     let mut database = self.intern.borrow_mut();
    //     let nset = database.name_result();
    //     f.debug(nset)
    // }

    pub fn statements(&self) -> impl Iterator<Item = (Label, Formula, Hypotheses)> + '_ {
        let database:&mut Database = &mut self.intern.borrow_mut();
        let sset = database.parse_result().clone();
        let nset = database.name_result().clone();
        sset.segments().iter().flat_map(|s| { s.into_iter() }).filter_map(move |sref| {
            match sref.statement_type() {
                StatementType::Axiom | StatementType::Provable => {
                    let name = sref.label();
                    let label = nset.lookup_label(name)?.atom;
                    let (formula, hyps) = Self::get_theorem_formulas_internal(database, label)?;
                    Some((label, formula, hyps))
                },
                _ => None,
            }
        }).collect::<Vec<_>>().into_iter()
    }

    /// Add a hypothesis step to a proof array
    pub fn build_proof_hyp(
        &self,
        label: Label,
        formula: Formula,
        stack_buffer: &mut Vec<u8>,
        arr: &mut ProofTreeArray,
    ) -> Option<usize> {
        let mut database = self.intern.borrow_mut();
        let sset = database.parse_result().clone();
        let nset = database.name_result().clone();
        let token = nset.atom_name(label);
        let address = nset.lookup_label(token)?.address;
        let range = formula.append_to_stack_buffer(stack_buffer, &sset, &nset);
        Some(arr.build(address, Default::default(), stack_buffer, range))
    }

    /// Add a normal step to a proof array
    pub fn build_proof_step(
        &self,
        label: Label,
        formula: Formula,
        mand_hyps: Vec<usize>,
        substitutions: &Substitutions,
        stack_buffer: &mut Vec<u8>,
        arr: &mut ProofTreeArray,
    ) -> Option<usize> {
        let mut database = self.intern.borrow_mut();
        let sset = database.parse_result().clone();
        let nset = database.name_result().clone();
        let scope = database.scope_result().clone();
        let token = nset.atom_name(label);
        let address = nset.lookup_label(token)?.address;
        let range = formula.append_to_stack_buffer(stack_buffer, &sset, &nset);
        let mut hyps = vec![];
        let frame = scope.get(token).unwrap();
        for hyp in frame.hypotheses.iter() {
            if let Hyp::Floating(sa, _, _) = hyp {
                let sref = sset.statement(*sa);
                let label = nset
                    .lookup_label(sref.label())
                    .map_or(Label::default(), |l| l.atom);
                let formula = &substitutions[label];
                let proof_tree_index = formula.build_syntax_proof::<usize, Vec<usize>>(
                    stack_buffer,
                    arr,
                    &sset,
                    &nset,
                    &scope,
                );
                hyps.push(proof_tree_index);
            }
        }
        hyps.extend(mand_hyps);
        Some(arr.build(address, hyps, stack_buffer, range))
    }

    pub fn export_mmp<W: Write>(&self, theorem: Label, arr: &ProofTreeArray, out: &mut W) {
        let mut database = self.intern.borrow_mut();
        let sset = database.parse_result().clone();
        let nset = database.name_result().clone();
        let scope = database.scope_result().clone();
        let thm_label = nset.atom_name(theorem);
        export_mmp_proof_tree(&sset, &nset, &scope, thm_label, arr, out).unwrap();
    }
}

impl Display for Symbol {
    fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(as_str(
            db.intern.borrow_mut().name_result().atom_name(*self),
        ))
    }
}

// Label and Symbol are actually both synonyms for `Atom`, no need to duplicate definition.
//
// impl Display for Label {
// 	fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
// 		f.write_str(&db.label_str(self))
// 	}
// }

impl Display for Formula {
    fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        let mut database = db.intern.borrow_mut();
        let sset = database.parse_result().clone();
        let nset = database.name_result().clone();
        for symbol in self.iter(&sset, &nset) {
            f.write_fmt(format_args!(" {}", as_str(nset.atom_name(symbol))))?;
        }
        Ok(())
    }
}

// impl Display for Substitutions {
//     fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
//         for (l,fmla) in self.0.iter() {
//             f.write_str("\n    ")?;
//             l.format(f, db)?;
//             f.write_str(" => ")?;
//             fmla.format(f, db)?;
//         }
//         Ok(())
//     }
// }

// impl Display for Box<Substitutions> {
//     fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
//         self.deref().format(f, db)
//     }
// }