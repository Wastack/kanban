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
        index: usize,
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
    /// Show issues
    Show {
        /// Specify what issue to show. If omitted, then it shows everything
        what: Option<ShowCategory>
    },
    /// Change priority (order) of issues
    Prio {
        /// Action (and direction) to take on the issue
        command: PrioCommand,

        /// Index of the issue to be moved
        index: usize,
    }
}

pub(crate) enum PrioCommand {
    /// Top of its category
    Top,
    /// Bottom of its category
    Bottom,
    /// 1 entry up in its category
    Up,
    /// 1 entry down in its category
    Down
}


impl FromStr for PrioCommand {
    // TODO proper error handling?
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "top" => Ok(PrioCommand::Top),
            "bottom" => Ok(PrioCommand::Bottom),
            "up" => Ok(PrioCommand::Up),
            "down" => Ok(PrioCommand::Down),
            _ => Err(String::from("unknown priority command"))
        }
    }
}


pub(crate) enum ShowCategory {
    // Show all the done tickets
    Done,
}


impl FromStr for ShowCategory {
    // TODO proper error handling?
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "done" => Ok(ShowCategory::Done),
            _ => Err(String::from("unknown category to show"))
        }
    }
}

impl FromStr for State {
    // TODO proper error handling?
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