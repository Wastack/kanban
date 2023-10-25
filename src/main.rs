mod controllers;
mod application;
mod presenters;
mod storage;
mod editor;

use clap::Parser;
use crate::controllers::{Command, PrioCommand, ShowCategory};
use crate::application::issue::{Described, Description, elapsed_time_since_epoch, Issue, State, Stateful};
use crate::application::ports::presenter::Presenter;
use crate::presenters::stdoutrenderer::{OnlyDoneStdOutRenderer, TabularTextRenderer};
use crate::storage::{home_file_storage, Storage};
use application::usecase::usecase::{AddUseCase, UseCase};


fn main() {
    let root = controllers::RootCli::parse();

    let use_case_callbacks: Vec<fn(_) -> Option<Box<dyn UseCase>>>  = vec![AddUseCase::call];


    let found = use_case_callbacks.iter().find_map(|&callback| {
        if root.command.is_none() {
            return None
        }

        let use_case = callback(root.command.clone().unwrap());
        if let Some(use_case) = use_case {
            Some(use_case)
        } else {
            None
        }
    });

    // TODO: here store the use_case for undo purposes

    // Legacy code structure
    // TODO: move this to UseCase
    if found.is_none() {
        let storage = home_file_storage();
        let mut board = storage.load();

        let mut board_changed= false;

        match root.command {
            Some(Command::Add{description, state}) => {
                let description = String::from(description.trim());

                board.issues.insert(0, Issue{
                    description: Description(description),
                    state: match state {
                        None => State::Open,
                        Some(s) => s,
                    },
                    time_created: elapsed_time_since_epoch(),
                });

                board_changed = true;
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
                let edited_description = editor::open_editor(issue.description().to_str())
                    .expect("preparing editor failed");

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


}




