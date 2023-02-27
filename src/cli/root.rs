use std::str::FromStr;
use clap::{Parser, Subcommand};
use crate::model::issue::State;

/// Sema cli is a utility to tweak SEMA resources.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct RootCli {
    #[clap(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Adds a new issue
    Add {
        /// A text that describes the issue
        description: String,

        /// Initial state of the ticket
        state: Option<State>,
    },
    /// Deletes an issue. This makes the indexes reassigned!
    Delete {
        // Index of the issue to delete
        index: u32,
    },
    /// Move an issue to a new state
    Move {
        // Index of the issue to delete
        index: u32,
        // New state to apply to the issue
        state: State,
    }
}

impl FromStr for State {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "analysis" => Ok(State::Analysis),
            "open" => Ok(State::Open),
            "progress" => Ok(State::InProgress),
            "review" => Ok(State::Review),
            "done" => Ok(State::Done),
            _ => Err(String::from("unknown state")),
        }
    }
}