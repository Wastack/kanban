mod controllers;
mod application;
mod adapters;

use clap::Parser;
use application::ports::issue_storage::IssueStorage;
use crate::controllers::{Command, PrioCommand};
use crate::application::issue::{State};
use crate::application::ports::presenter::Presenter;
use crate::adapters::presenters::stdoutrenderer::{TabularTextRenderer};
use application::usecase::add::{AddUseCase};
use crate::adapters::editors::os_default_editor::OsDefaultEditor;
use crate::application::ports::editor::Editor;
use crate::application::usecase::delete::DeleteUseCase;
use crate::application::usecase::edit::EditUseCase;
use crate::application::usecase::get::GetUseCase;
use crate::application::usecase::prio::PrioUseCase;
use crate::application::usecase::r#move::MoveUseCase;


fn main() {
    let root = controllers::RootCli::parse();

    match root.command {
        Some(Command::Add{description, state}) => {
            AddUseCase::default().execute(&description, state.unwrap_or(State::Open));
        },
        Some(Command::Delete{index}) => {
            DeleteUseCase::default().execute(&index);
        },
        Some(Command::Move{indices, state}) => {
            MoveUseCase::default().execute(&indices, &state);
        },
        Some(Command::Edit{index}) => {
            EditUseCase::default().execute(index);
        },
        Some(Command::Prio{command, index}) => {
            // TODO up is broken. Test?
            PrioUseCase::default().execute(index, command);
        },
        None => {
            GetUseCase::default().execute()
        },
    }

}




