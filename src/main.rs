#![feature(result_option_inspect)]

extern crate core;

mod application;
mod adapters;

use clap::Parser;
use adapters::controllers;
use application::ports::issue_storage::IssueStorage;
use adapters::controllers::{Command, PrioCommand};
use crate::application::issue::State;
use crate::application::ports::presenter::Presenter;
use crate::adapters::presenters::stdoutrenderer::TabularTextRenderer;
use application::usecase::add::AddUseCase;
use crate::adapters::editors::os_default_editor::OsDefaultEditor;
use crate::adapters::storages::FileStorage;
use crate::application::ports::editor::Editor;
use crate::application::usecase::delete::DeleteUseCase;
use crate::application::usecase::edit::EditUseCase;
use crate::application::usecase::get::GetUseCase;
use crate::application::usecase::prio::PrioUseCase;
use crate::application::usecase::r#move::MoveUseCase;
use crate::application::usecase::undo::UndoUseCase;

impl Default for Box<dyn IssueStorage> {
    fn default() -> Self {
        return Box::new(FileStorage::default())
    }
}

impl Default for Box<dyn Presenter> {
    fn default() -> Self {
        return Box::new(TabularTextRenderer::default())
    }
}

impl Default for Box<dyn Editor> {
    fn default() -> Self {
        return Box::new(OsDefaultEditor::default())
    }
}

fn main() {
    let root = controllers::RootCli::parse();

    match root.command {
        Some(Command::Add{description, state}) => {
            AddUseCase::default().execute(
                &description,
                state.unwrap_or(State::Open));
        },
        Some(Command::Delete{index}) => {
            DeleteUseCase::default().execute(&index);
        },
        Some(Command::Move{indices, state}) => {
            MoveUseCase::default().execute(&indices, state);
        },
        Some(Command::Edit{index}) => {
            let _ = EditUseCase::default().execute(index);
        },
        Some(Command::Prio{command, index}) => {
            PrioUseCase::default().execute(index, command);
        },
        Some(Command::Undo) => {
            let _ = UndoUseCase::default().execute();
        },
        None => {
            GetUseCase::default().execute()
        },
    }

}




