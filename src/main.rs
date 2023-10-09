mod cli;
mod model;
mod render;
mod storage;
mod editor;

use home::home_dir;
use clap::Parser;
use crate::cli::{Commands, ShowCategory, PrioCommand};
use crate::model::board::Board;
use crate::model::issue::{Described, Description, elapsed_time_since_epoch, Issue, State, Stateful};
use crate::render::render::Renderer;
use crate::render::stdoutrenderer::{OnlyDoneStdOutRenderer, TabularTextRenderer};
use crate::storage::{Storage};


fn main() {
    let root = cli::RootCli::parse();

    let storage = storage::FileStorage{
        source: home_dir().expect("Failed to get home directory")
            .join(".kanban").into(),
    };
    let mut board = storage.load();

    let mut board_changed= false;

    match root.command {
        Some(Commands::Add{description, state}) => {
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
        Some(Commands::Delete{mut index}) => {
            // Sort the indices in descending order,
            // so that each removal does not affect the next index.
            index.sort_unstable_by(|a, b| b.cmp(a));

            for &i in &index {
                // This will panic if out of range. Is that good?
                board.issues.remove(i);
            }

            board_changed = true;
        },
        Some(Commands::Move{indices, state}) => {
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
        Some(Commands::Edit{index}) => {
            let issue = board.issues.get_mut(index)
                .expect("did not find issue with index");
            let edited_description = editor::open_editor(issue.description().to_str())
                .expect("preparing editor failed");

            let description = issue.description_mut();
            description.0 = edited_description;

            board_changed = true;
        },
        Some(Commands::Show{what}) => {
            match what {
                Some(ShowCategory::Done) => {
                    let out = OnlyDoneStdOutRenderer::default().render_board(&board);
                    println!("{}", out)
                },
                None => {
                    // TODO show all done stories this case?
                    let out = TabularTextRenderer::default().render_board(&board);
                    println!("{}", out)
                }
            }
        },
        Some(Commands::Prio{command, index}) => {
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
            let out = TabularTextRenderer::default().render_board(&board);
            println!("{}", out)
        },
    }

    if board_changed {
        storage.save(&board)
    }
}




