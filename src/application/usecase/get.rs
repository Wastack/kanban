use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct GetUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P
}

impl<I: IssueStorage, P: Presenter> GetUseCase<I, P> {
    pub(crate) fn execute(&self) {
        let board = self.storage.load();
        self.presenter.render_board(&board);
    }
}