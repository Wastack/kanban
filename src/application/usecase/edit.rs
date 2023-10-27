use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::{Editor, unwrap_or_return};
use crate::application::domain::issue::Described;


#[derive(Default)]
pub(crate) struct EditUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>,
    editor: Box<dyn Editor>,
}

impl EditUseCase {
    pub(crate) fn execute(&mut self, index: usize) {
        let mut board = self.storage.load();

        let issue = unwrap_or_return!(board
            .get_issue_mut(index)
            .inspect_err(|e| {
                self.presenter.render_error(&e);
            }));

        let edited_description = unwrap_or_return!(self.editor
            .open_editor_with(
                issue.description().as_str())
            .inspect_err(|e| self.presenter.render_error(&e)));

        issue.description_mut().set(&edited_description);

        self.storage.save(&board);
        self.presenter.render_board(&board);
    }
}