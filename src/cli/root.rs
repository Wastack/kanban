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
    },
    /// Edit the description of an issue with $EDITOR (defaults to vim)
    Edit {
        // Index of the issue to edit
        index: u32,
    }
}
