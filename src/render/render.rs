use crate::model::board::Board;

pub trait Renderer {
    fn render_board(&self, board: &Board) -> String;
}
