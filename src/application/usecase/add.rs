use crate::application::{elapsed_time_since_epoch, Issue, State};
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

        board.issues.insert(0, Issue{
            description: Description::from(description),
            state,
            time_created: elapsed_time_since_epoch(),
        });

        self.storage.save(&board);

        self.presenter.render_board(&board)
    }

}