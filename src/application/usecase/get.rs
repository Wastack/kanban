use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct GetUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl GetUseCase {
    pub(crate) fn execute(&self) {
        let board = self.storage.load();
        self.presenter.render_board(&board);
    }
}