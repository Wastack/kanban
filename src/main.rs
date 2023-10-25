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
use crate::adapters::storages::FileStorage;
use crate::application::ports::editor::Editor;
use crate::application::usecase::delete::DeleteUseCase;
use crate::application::usecase::edit::EditUseCase;
use crate::application::usecase::r#move::MoveUseCase;


fn main() {
    let root = controllers::RootCli::parse();

    let storage = FileStorage::default();
    let mut board = storage.load();

    let mut board_changed= false;

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
            // TODO Out Of Range
            match command {
                PrioCommand::Top => board.prio_top_in_category(index as usize),
                PrioCommand::Bottom => board.prio_bottom_in_category(index as usize),
                PrioCommand::Up => board.prio_up_in_category(index as usize),
                PrioCommand::Down => board.prio_down_in_category(index as usize),
            }

            board_changed = true;
        },
        None => {
            TabularTextRenderer::default().render_board(&board);
        },
    }

    if board_changed {
        storage.save(&board);
        TabularTextRenderer::default().render_board(&board);
    }



}




