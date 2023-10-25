mod controllers;
mod application;
mod adapters;

use clap::Parser;
use application::ports::issue_storage::IssueStorage;
use crate::controllers::{Command, PrioCommand, ShowCategory};
use crate::application::issue::{Described, State, Stateful};
use crate::application::ports::presenter::Presenter;
use crate::adapters::presenters::stdoutrenderer::{OnlyDoneStdOutRenderer, TabularTextRenderer};
use crate::adapters::storages::home_file_storage;
use application::usecase::add::{AddUseCase};
use crate::adapters::editors::os_default_editor::OsDefaultEditor;
use crate::application::ports::editor::Editor;


fn main() {
    let root = controllers::RootCli::parse();

    let storage = home_file_storage();
    let mut board = storage.load();

    let mut board_changed= false;

    match root.command {
        Some(Command::Add{description, state}) => {
            AddUseCase::execute(description, state.unwrap_or(State::Open));
        },
        Some(Command::Delete{mut index}) => {
            // Sort the indices in descending order,
            // so that each removal does not affect the next index.
            index.sort_unstable_by(|a, b| b.cmp(a));

            for &i in &index {
                // This will panic if out of range. Is that good?
                board.issues.remove(i);
            }

            board_changed = true;
        },
        Some(Command::Move{indices, state}) => {
            // Check if all indices are valid
            if !indices.iter().all(|i|*i < board.issues.len()) {
                if indices.len() > 1 {
                    panic!("at least one of the indices specified are out of range")
                } else {
                    panic!("index out of range")
                }
            }

            for index in indices {
                let current_state = board.issues[index].state_mut();

                if *current_state != state{
                    *current_state = state;

                    if state == State::Done {
                        board.prio_top_in_category(index);
                    }
                }
            }

            board_changed = true;
        },
        Some(Command::Edit{index}) => {
            let issue = board.issues.get_mut(index)
                .expect("did not find issue with index");
            // TODO use port instead
            let editor = OsDefaultEditor{};
            let edited_description = editor.open_editor_with(issue.description().to_str())
                .expect("preparing editors failed");

            let description = issue.description_mut();
            description.0 = edited_description;

            board_changed = true;
        },
        Some(Command::Show{what}) => {
            match what {
                Some(ShowCategory::Done) => {
                    OnlyDoneStdOutRenderer::default().render_board(&board);
                },
                None => {
                    // TODO show all done stories this case?
                    TabularTextRenderer::default().render_board(&board);
                }
            }
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




