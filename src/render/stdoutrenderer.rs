use crate::model::board::Board;
use crate::model::issue::{Described, Stateful};
use crate::render::render::Renderer;


#[derive(Default)]
pub struct StdOutRenderer {}


impl Renderer for StdOutRenderer {
    fn render_board(&self, board: &Board) -> String {
        board.issues.iter().enumerate().fold(String::new(), |current, (index, issue) | {
            current + &format!("{}\t{}\t{:?}\n", index, issue.description(), issue.state())
        })
    }
}