extern crate core;

mod cli;
mod model;
mod render;

use std::fmt::{Display, Formatter};
use std::io;
use std::num::ParseIntError;
use std::process::exit;
use crate::cli::Commands;


fn main() {
    if let Err(e) = process() {
        println!("{}", e);
        exit(1);
    }
}

fn process() -> CliResult<()> {
    let root = cli::RootCli::parse();
    Config::initialize(root.verbose)?;

    match root.command {
        Commands::Add(description, state) => {
            // TODO add issue with description and state
        },
        None => {
            // TODO list current issues
        },
    }

    Ok(())
}


