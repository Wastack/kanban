use std::error;
use crate::application::board::Board;

pub trait Presenter {
    fn render_board(&self, board: &Board);
    fn render_error(&mut self, err: &dyn error::Error);
}


