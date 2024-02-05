use crate::application::board::Board;
use crate::application::domain::error::DomainError;
use crate::application::Issue;

pub trait Presenter {
    fn render_board(&mut self, board: &Board<Issue>);
    fn render_error(&mut self, err: &DomainError);
}


