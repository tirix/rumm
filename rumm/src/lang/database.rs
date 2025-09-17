use crate::error::{Error, Result};
use crate::lang::Display;
use crate::tactics::{TacticsResult, TacticsError};

use colored::*;
use metamath_rs::grammar::FormulaToken;
use core::cell::RefCell;
use core::fmt::Formatter;
use metamath_rs::as_str;
use metamath_rs::database::DbOptions;
use metamath_rs::diag::StmtParseError;
use metamath_rs::formula::Substitutions;
use metamath_rs::proof::ProofTreeArray;
use metamath_rs::verify::ProofBuilder;
use metamath_rs::Database;
use metamath_rs::Formula;
use metamath_rs::Label;
use metamath_rs::StatementType;
use metamath_rs::Symbol;
use std::ops::Deref;
use std::io::Write;
use std::sync::Arc;

/// A type for representing theorem essential hypotheses: a label and the corresponding formula.
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
    pub fn parse(&mut self, filename: &str) -> Result {
        let mut database = self.intern.borrow_mut();

        print!("Loading \"{}\" ... ", filename);
        database.parse(filename.to_string(), Vec::new());
        let err = database.diag_notations();
        if !err.is_empty() {
            return Err(Error::DBError(filename.to_string(), err).into());
        }
        println!("{}", "ok".green());

        print!("Building \"{}\" grammar... ", filename);
        database.grammar_pass();
        let gerr = database.diag_notations();
        if !gerr.is_empty() {
            return Err(Error::DBError(filename.to_string(), gerr).into());
        }
        println!("{}", "ok".green());

        print!("Parsing \"{}\"... ", filename);
        database.stmt_parse_pass();
        let gerr = database.diag_notations();
        if !gerr.is_empty() {
            return Err(Error::DBError(filename.to_string(), gerr).into());
        }
        println!("{}", "ok".green());
        //let testx = self.get_theorem_label("testx".to_string()).unwrap();
        //let (_f, ..) = self.get_theorem_formulas(testx).unwrap();
        //println!("TESTX {}", f.debug(self.database.borrow_mut().name_result()));
        Ok(())
    }

    pub fn get_symbol(&self, name: String) -> Option<Symbol> {
        let database = self.intern.borrow();
        Some(database
            .name_result()
            .lookup_symbol(name.as_bytes())?
            .atom
        )
    }

    pub fn get_theorem_label(&self, name: String) -> Option<Label> {
        let database = self.intern.borrow();
        Some(database
            .name_result()
            .lookup_label(name.as_bytes())?.atom
        )
    }

    pub fn get_theorem_formulas(&self, label: Label) -> Option<(Formula, Hypotheses)> {
        let database = self.intern.borrow();
        let sref = database.statement_by_label(label)?;
        let formula = database.stmt_parse_result().get_formula(&sref)?.clone();
        let frame = database.get_frame(label)?;
        let hypotheses: Vec<(Label, Formula)> = frame.essentials().map(|(l,f)| { (l, f.clone()) }).collect();
        Some((formula, hypotheses.into_boxed_slice()))
    }

    pub fn parse_formula(&self, symbols: Vec<std::result::Result<FormulaToken, StmtParseError>>) -> std::result::Result<Formula, StmtParseError> {
        let database = self.intern.borrow();
        let grammar = database.grammar_result().clone();
        let nset = database.name_result();
        let convert_to_provable = false;
        grammar
            .parse_formula(&mut symbols.into_iter(), &grammar.typecodes(), convert_to_provable, nset)
    }

    pub fn ensure_type(&self, fmla: Formula, label: Label) -> TacticsResult<Formula> {
        let database = self.intern.borrow();
        let target_tc = database.label_typecode(label);
        let source_tc = fmla.get_typecode();
        if target_tc != source_tc {
            let grammar = database.grammar_result().clone();
            grammar.convert_typecode(fmla, target_tc).ok_or(TacticsError::WrongTypecode(source_tc, target_tc, label))
        } else {
            Ok(fmla)
        }
    }

    pub fn debug_formula<'a>(&'a self, f: &'a Formula) {
        let database = self.intern.borrow();
        println!("{:?}", f.as_ref(&database));
    }

    pub fn statements(&self, filter: impl Fn(bool, &[u8]) -> bool) -> impl Iterator<Item = (Label, Formula, Hypotheses)> + '_ {
        let database = self.intern.borrow();
        let nset = database.name_result().clone();
        let provable = database.grammar_result().provable_typecode();
        database.statements().filter_map(move |sref| {
            match sref.statement_type() {
                StatementType::Axiom | StatementType::Provable => {
                    let is_axiom = sref.statement_type() == StatementType::Axiom;
                    let name = sref.label();
                    if nset.get_atom(&sref.math_at(0)) == provable && filter(is_axiom, name) {
                        let label = nset.lookup_label(name)?.atom;
                        let (formula, hyps) = self.get_theorem_formulas(label)?;
                        Some((label, formula, hyps))
                    } else { None }
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
        let database = self.intern.borrow();
        let nset = database.name_result().clone();
        let token = nset.atom_name(label);
        let address = nset.lookup_label(token)?.address;
        let range = formula.as_ref(&database).append_to_stack_buffer(stack_buffer);
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
        let database = self.intern.borrow();
        let token = database.name_result().atom_name(label);
        let address = database.name_result().lookup_label(token)?.address;
        let range = formula.as_ref(&database).append_to_stack_buffer(stack_buffer);
        let frame = database.get_frame(label)?;
        let mut hyps = vec![];
        for label in frame.floating() {
            let formula = &substitutions.get(label).unwrap_or_else(|| {
                panic!("While building proof using {}: No substitution for {}", as_str(token), as_str(database.name_result().atom_name(label)));
            });
            let proof_tree_index = formula.as_ref(&database).build_syntax_proof::<usize, Vec<usize>>(
                stack_buffer,
                arr,
            );
            hyps.push(proof_tree_index);
        }
        hyps.extend(mand_hyps);
        Some(arr.build(address, hyps, stack_buffer, range))
    }

    pub fn export_mmp<W: Write>(&self, theorem: Label, arr: &ProofTreeArray, out: &mut W) {
        let database = self.intern.borrow();
        let thm_label = database.name_result().atom_name(theorem);
        database.export_mmp_proof_tree(thm_label, arr, out).unwrap();
    }
}

impl Display for Symbol {
    fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        let database = db.intern.borrow();
        f.write_str(as_str(
            database.name_result().atom_name(*self),
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
        let database = db.intern.borrow();
        std::fmt::Display::fmt(&self.as_ref(&database), f)
    }
}

impl Display for Substitutions {
    fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        let database = db.intern.borrow();
        std::fmt::Debug::fmt(&self.as_ref(&database), f)
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

impl Display for Box<Substitutions> {
    fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error> {
        self.deref().format(f, db)
    }
}