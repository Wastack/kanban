use nonempty_collections::NEVec;
use crate::application::domain::historized_board::HistorizedBoard;
use crate::application::domain::error::DomainError;
use crate::application::Issue;

pub trait Presenter {
    fn render_board(&mut self, board: &HistorizedBoard<Issue>);
    fn render_error(&mut self, err: &DomainError);

    fn render_errors(&mut self, errors: &NEVec<DomainError>) {
        for err in errors {
            self.render_error(err);
        }
    }
}


