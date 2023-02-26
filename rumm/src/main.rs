//! A Metamath Proof Assistant

mod context;
pub mod error;
mod lang;
pub mod parser;
pub mod script;
pub mod tactics;
use annotate_snippets::display_list::DisplayList;
use clap::{clap_app, crate_version};
use error::Result;
use script::Script;
use typed_arena::Arena;
use std::fs;

fn run() -> Result {
    let app = clap_app!(("rumm") =>
        (version: crate_version!())
        (about: "A tactics based proof language for Metamath")
        (@arg RMM_FILE: "Rumm file to load"));
    let matches = app.get_matches();
    let path = matches.value_of("RMM_FILE").unwrap_or("../set.rmm");
    let data = fs::read_to_string(path)?; // TODO use map_err to map into an error storing the file name for context.
    let mut script = Script::from_str(path.to_string(), &data)?;
    script.execute()
}

fn main() {
    let arena: Arena<String> = Arena::new();
    match run() {
        Ok(()) => println!("Done."),
        Err(error) => eprintln!("{}", DisplayList::from(error.to_snippet(&arena))),
    };
}
