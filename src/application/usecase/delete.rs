use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct DeleteUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl DeleteUseCase {
    pub(crate) fn execute(&self, indices: &[usize]) {
        let mut board = self.storage.load();

        // Sort the indices in descending order,
        // so that each removal does not affect the next index.
        let mut sorted_indices = indices.to_owned();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

        for &i in &sorted_indices {
            // This will panic if out of range. Is that good?
            board.issues.remove(i);
        }

        self.presenter.render_board(&board);
    }
}