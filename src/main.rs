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
use application::usecase::add::AddUseCase;
use crate::adapters::editors::os_default_editor::OsDefaultEditor;
use crate::adapters::presenters::stdoutrenderer::TabularTextRenderer;
use crate::adapters::storages::FileStorage;
use crate::adapters::time_providers::fake::FakeTimeProvider;
use crate::adapters::time_providers::simple::SimpleTimeProvider;
use crate::application::ports::editor::Editor;
use crate::application::usecase::delete::DeleteUseCase;
use crate::application::usecase::edit::EditUseCase;
use crate::application::usecase::get::GetUseCase;
use crate::application::usecase::prio::PrioUseCase;
use crate::application::usecase::r#move::MoveUseCase;
use crate::application::usecase::undo::UndoUseCase;

fn main() {
    let root = controllers::RootCli::parse();

    match root.command {
        Some(Command::Add{description, state}) => {
            AddUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, FakeTimeProvider>::default().execute(
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
            let _ = EditUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>, OsDefaultEditor>::default().execute(index);
        },
        Some(Command::Prio{command, index}) => {
            PrioUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute(index, command);
        },
        Some(Command::Undo) => {
            let _ = UndoUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute();
        },
        None => {
            GetUseCase::<FileStorage, TabularTextRenderer<SimpleTimeProvider>>::default().execute()
        },
    }

}




