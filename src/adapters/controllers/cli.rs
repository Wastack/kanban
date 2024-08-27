use std::str::FromStr;
use clap::{Parser, Subcommand};
use crate::adapters::editors::os_default_editor::OsDefaultEditor;
use crate::adapters::presenters::stdoutrenderer::TabularTextRenderer;
use crate::adapters::storages::FileStorage;
use crate::adapters::time_providers::simple::SimpleTimeProvider;
use crate::application::State;
use crate::application::usecase::add::AddUseCase;
use crate::application::usecase::delete::DeleteUseCase;
use crate::application::usecase::edit::EditUseCase;
use crate::application::usecase::get::GetUseCase;
use crate::application::usecase::flush::FlushUseCase;
use crate::application::usecase::prio::{BottomPriority, DownPriority, PriorityUseCase, TopPriority, UpPriority};
use crate::application::usecase::r#move::MoveUseCase;
use crate::application::usecase::undo::UndoUseCase;

/// Sema cli is a utility to tweak SEMA resources.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct RootCli {
    #[clap(subcommand)]
    pub(crate) command: Option<Command>,
}

impl RootCli {
    pub(crate) fn execute(self) {
        match self.command {
            Some(Command::Add{description, state}) => {
                AddUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, SimpleTimeProvider>::default().execute(
                    &description,
                    state.unwrap_or(State::Open));
            },
            Some(Command::Delete{index}) => {
                DeleteUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute(&index);
            },
            Some(Command::Move{indices, state}) => {
                MoveUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute(&indices, state);
            },
            Some(Command::Edit{index}) => {
                EditUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, OsDefaultEditor>::default().execute(index);
            },
            Some(Command::Prio{
                     command: PrioCommand::Top,
                     index
                 }) => {
                PriorityUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, TopPriority>::default().execute(index);
            },
            Some(Command::Prio{
                     command: PrioCommand::Bottom,
                     index
                 }) => {
                PriorityUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, BottomPriority>::default().execute(index);
            },
            Some(Command::Prio{
                     command: PrioCommand::Up,
                     index
                 }) => {
                PriorityUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, UpPriority>::default().execute(index);
            },
            Some(Command::Prio{
                     command: PrioCommand::Down,
                     index
                 }) => {
                PriorityUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, DownPriority>::default().execute(index);
            },
            Some(Command::Undo) => {
                UndoUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute();
            },
            Some(Command::Flush) => {
                FlushUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute();
            },
            None => {
                GetUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute()
            },
        }
    }
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
    /// Flush deletes every issues that are not done
    Flush,
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