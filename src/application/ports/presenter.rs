use crate::application::board::Board;
use crate::application::domain::error::DomainError;

pub trait Presenter {
    fn render_board(&self, board: &Board);
    fn render_error(&mut self, err: &DomainError);
}


