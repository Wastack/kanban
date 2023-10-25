use crate::application::board::Board;

pub trait Presenter {
    fn render_board(&self, board: &Board);
}


