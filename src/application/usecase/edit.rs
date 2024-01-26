use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::{Editor};
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::{EditHistoryElement, UndoableHistoryElement};
use crate::application::domain::issue::Described;


#[derive(Default)]
pub(crate) struct EditUseCase<I: IssueStorage, P: Presenter, E: Editor> {
    storage: I,
    presenter: P,
    editor: E,
}

impl<I: IssueStorage, P: Presenter, E: Editor> EditUseCase<I, P, E> {
    pub(crate) fn execute(&mut self, index: usize) -> DomainResult<()> {
        let mut board = self.storage.load();

        let issue =(board
            .get_issue(index)
            .inspect_err(|e| {
                self.presenter.render_error(&e);
            }))?;

        let original_description = String::from(issue.description.as_str());

        let edited_description = self.editor
            .open_editor_with(
                issue.description().as_str())
            .inspect_err(|e| self.presenter.render_error(&e))
            .map_err(|e|DomainError::new(&e.to_string()))?;

        let issue = board.get_issue_mut(index)?;
        issue.description_mut().set(&edited_description);

        board.history_mut().push(UndoableHistoryElement::Edit(
            EditHistoryElement {
                original_description,
            }
        ));



        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Error, ErrorKind};
    use crate::{Editor, EditUseCase, IssueStorage, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::{Board, Issue};
    use crate::application::usecase::tests_common::tests::then_result;

    #[test]
    fn test_execute_successful_editing() {
        let mut edit_use_case = given_edit_usecase_with::<TestEditor>(
            Board::default().with_4_typical_issues(),
        );

        let result = edit_use_case.execute(2);

        then_stored_issue_of_the(&edit_use_case)
            .assert_description_edited()
            .assert_other_issues_did_not_change();
        then_result(&result)
            .assert_succeeded();
    }

    #[test]
    fn test_editing_issue_with_index_out_of_range() {
        let mut edit_use_case = given_edit_usecase_with::<TestEditor>(
            Board::default().with_4_typical_issues(),
        );

        let result = edit_use_case.execute(4);

        then_edited_board(&edit_use_case)
            .assert_has_original_issues();
        then_result(&result)
            .assert_failed_with("Index out of range");
    }

    #[test]
    fn test_editor_closes_abruptly() {
        let mut edit_use_case = given_edit_usecase_with::<CloseAbruptlyEditor>(
            Board::default().with_4_typical_issues(),
        );

        let result = edit_use_case.execute(4);

        then_edited_board(&edit_use_case)
            .assert_issue_count(4)
            .assert_has_original_issues();
        then_result(&result)
            .assert_failed();
    }

    fn then_edited_board<E: Editor>(sut: &EditUseCase<MemoryIssueStorage, NilPresenter, E>) -> Board {
        sut.storage.load()
    }

    fn then_stored_issue_of_the<E: Editor>(sut: &EditUseCase<MemoryIssueStorage, NilPresenter, E>) -> Issue {
        let board = sut.storage.load();
        let issue = board.get_issue(2);
        assert!(issue.is_ok());
        issue.unwrap().clone()
    }

    impl Issue {
        fn assert_description_edited(&self) -> &Self {
            assert_eq!(self.description.as_str(), "Edited: Task inserted second");
            self
        }

        fn assert_other_issues_did_not_change(&self) -> &Self {
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

    #[derive(Default)]
    struct CloseAbruptlyEditor {}

    impl Editor for CloseAbruptlyEditor {
        fn open_editor_with(&self, _text: &str) -> Result<String, Error> {
            Err(Error::new(ErrorKind::Other, "Bamm. I'm dead"))
        }
    }

    fn given_edit_usecase_with<E: Editor + Default>(board: Board) -> EditUseCase<MemoryIssueStorage, NilPresenter, E> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        EditUseCase {
            storage,
            ..Default::default()
        }
    }
}
