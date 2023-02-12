//! A Metamath Proof Assistant

mod context;
pub mod error;
mod lang;
pub mod parser;
pub mod script;
pub mod tactics;
use anyhow::Context;
use clap::{clap_app, crate_version};
use error::Result;
use script::Script;
use std::fs;

fn run() -> Result {
    let app = clap_app!(("rumm") =>
        (version: crate_version!())
        (about: "A tactics based proof language for Metamath")
        (@arg RMM_FILE: "Rumm file to load"));
    let matches = app.get_matches();
    let path = matches.value_of("RMM_FILE").unwrap_or("../set.rmm");
    let data = fs::read_to_string(path)
        .with_context(|| format!("could not read file `{}`", path))?;
    let mut script = Script::from_str(path.to_string(), &data)?;
    script.execute()
}

fn main() {
    match run() {
        Ok(()) => println!("Done."),
        Err(error) => eprintln!("Error: {:?}", error),
    };
}
