mod cli;
mod model;
mod render;
mod storage;

use home::home_dir;
use clap::Parser;
use crate::cli::{Commands};
use crate::model::board::Board;
use crate::model::issue::{Description, Issue, State, Stateful};
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
            board.issues.push(Issue{
                description: Description(description),
                state: match state {
                    None => State::Open,
                    Some(s) => s,
                }
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
        }
        None => {
            let out = TabularTextRenderer::default().render_board(&board);
            println!("{}", out)
        },
    }

    if board_changed {
        storage.save(&board)
    }
}




