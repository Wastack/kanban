use crate::model::Board;

pub trait  Renderer {
    fn render_board(board: &Board);
}
