use crate::application::{Issue, State};
use crate::application::issue::Description;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct AddUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl AddUseCase {
    pub(crate) fn execute(&mut self, description: &str, state: State) {
        let mut board = self.storage.load();

        board.insert_issue(Issue::new(
            Description::from(description),
            state,
        ));

        self.storage.save(&board);

        self.presenter.render_board(&board)
    }

}