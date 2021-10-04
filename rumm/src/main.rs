//! A Metamath Proof Assistant

mod context;
pub mod error;
mod lang;
pub mod parser;
pub mod script;
pub mod tactics;
//use simple_logger::SimpleLogger;
use error::{Error, Result};
use script::Script;
use std::fs;
use std::str::FromStr;

fn run() -> Result {
    let data = fs::read_to_string("examples/set.rmm")?;
    let mut script = Script::from_str(&data)?;
    script.execute()
}

fn main() {
    //SimpleLogger::new().init().unwrap();
    match run() {
        Ok(()) => println!("Done."),
        Err(error) => eprintln!("Error: {:?}", error),
    };
}
