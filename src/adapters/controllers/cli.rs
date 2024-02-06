use std::str::FromStr;
use clap::{Parser, Subcommand};
use crate::application::State;

/// Sema cli is a utility to tweak SEMA resources.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct RootCli {
    #[clap(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Subcommand, Clone)]
pub(crate) enum Command {
    /// Add new issue
    Add {
        /// A text that describes the issue
        description: String,

        /// Initial state of the ticket
        state: Option<State>,
    },
    /// Deletes an issue. This makes the indexes reassigned!
    Delete {
        // Index of the issue to delete
        index: Vec<usize>,
    },
    /// Move issue to a new state
    Move {
        // New state to apply to the issue
        state: State,

        // One or multiple indices of issues that you move
        indices: Vec<usize>,
    },
    /// Edit the description of an issue with $EDITOR (defaults to vim)
    Edit {
        // Index of the issue to edit
        index: usize,
    },
    /// Change priority (order) of issues
    Prio {
        /// Action (and direction) to take on the issue
        command: PrioCommand,

        /// Index of the issue to be moved
        index: usize,
    },
    /// Undo last action that changed the state
    Undo,
}

#[derive(Clone)]
pub(crate) enum PrioCommand {
    /// Top of its category
    Top,
    /// Bottom of its category
    Bottom,
    /// 1 entry up in its category
    Up,
    /// 1 entry down in its category
    Down,
}


impl FromStr for PrioCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "top" => Ok(PrioCommand::Top),
            "bottom" => Ok(PrioCommand::Bottom),
            "up" => Ok(PrioCommand::Up),
            "down" => Ok(PrioCommand::Down),
            // This error message is presented by clap directly
            _ => Err(String::from("unknown priority command"))
        }
    }
}


impl FromStr for State {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
               "open" => Ok(State::Open),
               "review" => Ok(State::Review),
               "done" => Ok(State::Done),
                // This error message is presented by clap directly
               _ => Err(String::from("unknown state")),
        }
    }
}