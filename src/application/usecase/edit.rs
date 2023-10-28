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

#[cfg(test)]
mod tests {
    use std::io::Error;
    use crate::application::usecase::usecase_test::tests::board_with_4_issues;
    use crate::{Editor, EditUseCase, IssueStorage, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::{Board, Issue};

    #[test]
    fn test_execute_successful_editing() {
        let mut edit_usecase = given_edit_usecase_with(
            board_with_4_issues(),
        );

        edit_usecase.execute(1);

        then_edited_issue(&edit_usecase)
            .has_edited_description()
            .but_nothing_else_changed();
    }

    #[test]
    fn test_editing_issue_with_index_out_of_range() {
        let mut sut = given_edit_usecase_with(
            board_with_4_issues(),
        );

        sut.execute(4);

        then_edited_board(&mut sut)
            .did_not_change();
    }

    fn then_edited_board(sut: &mut EditUseCase) -> Board {
        sut.storage.load()
    }

    fn then_edited_issue(sut: &EditUseCase) -> Issue {
        let board = sut.storage.load();
        let issue = board.get_issue(1);
        assert!(issue.is_ok());
        issue.unwrap().clone()
    }

    impl Issue {
        fn has_edited_description(&self) -> &Self {
            assert_eq!(self.description.as_str(), "Edited: Second task");
            self
        }

        fn but_nothing_else_changed(&self) -> &Self {
            assert_eq!(self.state, State::Review);
            assert_eq!(self.time_created, 1698397490);
            self
        }
    }

    #[derive(Default)]
    struct TestEditor { }

    impl Editor for TestEditor {
        fn open_editor_with(&self, text: &str) -> Result<String, Error> {
            Ok(format!("Edited: {}", text))
        }
    }

    fn given_edit_usecase_with(board: Board) -> EditUseCase {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        EditUseCase {
            storage: Box::new(storage),
            presenter: Box::new(NilPresenter::default()),
            editor: Box::new(TestEditor::default())
        }
    }
}
