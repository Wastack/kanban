mod cli;
mod model;
mod render;
mod storage;

use home::home_dir;
use clap::Parser;
use crate::cli::Commands;
use crate::model::board::Board;
use crate::model::issue::{Description, Issue, State};
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
                    Some(s) => match s.to_lowercase().as_str() {
                        "analysis" => State::Analysis,
                        "blocked" => State::Blocked,
                        "open" => State::Open,
                        "progress" => State::InProgress,
                        "review" => State::Review,
                        "done" => State::Done,
                        _ => panic!("unknown state"),
                    }
                }
            });

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




