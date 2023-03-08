mod cli;
mod model;
mod render;
mod storage;
mod editor;

use home::home_dir;
use clap::Parser;
use crate::cli::{Commands};
use crate::model::board::Board;
use crate::model::issue::{Described, Description, elapsed_time_since_epoch, Issue, State, Stateful};
use crate::render::render::Renderer;
use crate::render::stdoutrenderer::TabularTextRenderer;
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
        Some(Commands::Delete{index}) => {
            // This will panic if out of range. Is that good?
            board.issues.remove(index as usize);

            board_changed = true;
        },
        Some(Commands::Move {index, state}) => {
            let current_state = board.issues[index as usize].state_mut();
            *current_state = state;

            board_changed = true;
        },
        Some(Commands::Edit {index}) => {
            let issue = board.issues.get_mut(index as usize)
                .expect("did not find issue with index");
            let edited_description = editor::open_editor(issue.description().to_str())
                .expect("preparing editor failed");

            let description = issue.description_mut();
            description.0 = edited_description;

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




