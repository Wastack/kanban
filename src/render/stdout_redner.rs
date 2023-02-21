use crate::model::Board;
use crate::render::render::Renderer;

pub struct StdOutRenderer {}


impl Renderer for StdOutRenderer {
    fn render_board(board: &Board) {
        for (i, x) in board.issues.iter().enumerate() {
            println!("{}\t{}\t{:?}", i, x.description(), i.state)
        }
    }
}