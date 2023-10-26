use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::Editor;
use crate::application::domain::issue::Described;


#[derive(Default)]
pub(crate) struct EditUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>,
    editor: Box<dyn Editor>,
}

impl EditUseCase {
    pub(crate) fn execute(&self, index: usize) {
        let mut board = self.storage.load();

        let issue = board.issues.get_mut(index)
            .expect("did not find issue with index");
        let edited_description = self.editor.open_editor_with(issue.description().as_str())
            .expect("preparing editors failed");

        issue.description_mut().set(&edited_description);

        self.storage.save(&board);
        self.presenter.render_board(&board);
    }
}