#![feature(extract_if)]

extern crate core;

mod application;
mod adapters;

use clap::Parser;
use adapters::controllers;

fn main() {
    let command = controllers::RootCli::parse();
    command.execute();
}




