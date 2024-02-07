use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::{Editor};
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::{EditHistoryElement, UndoableHistoryElement};


#[derive(Default)]
pub(crate) struct EditUseCase<I: IssueStorage, P: Presenter, E: Editor> {
    storage: I,
    presenter: P,
    editor: E,
}

impl<I: IssueStorage, P: Presenter, E: Editor> EditUseCase<I, P, E> {
    pub(crate) fn execute(&mut self, index: usize) -> DomainResult<()> {
        let mut board = self.storage.load();

        let id = board.find_entity_id_by_index(index)
            .inspect_err(|e| {
                self.presenter.render_error(&e);
            })?;

        let entitiy = board.get(id);

        let original_description = String::from(entitiy.description.as_str());

        let edited_description = self.editor
            .open_editor_with( entitiy.description.as_str())
            .map_err(|e|DomainError::from(e))
            .inspect_err(|e| self.presenter.render_error(e))?;

        let issue = board.get_mut(id);
        issue.description.set(&edited_description);

        board.history.push(UndoableHistoryElement::Edit(
            EditHistoryElement {
                original_description,
                index,
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
    use crate::application::domain::error::DomainError;
    use crate::application::issue::Entity;

    #[test]
    fn test_execute_successful_editing() {
        let mut edit_use_case = given_edit_usecase_with::<TestEditor>(
            Board::default().with_4_typical_issues(),
        );

        let result = edit_use_case.execute(2);

        let stored_board = edit_use_case.storage.load();
        then_stored_issue_of_the(&stored_board)
            .assert_description_edited()
            .assert_other_issues_did_not_change();

        let presented_board = edit_use_case.presenter.last_board_rendered.expect("Expected a board to be presented");
        assert_eq!(presented_board, stored_board, "Expected stored and presented board to be equal");

        assert!(matches!(result, Ok(_)))
    }

    #[test]
    fn test_editing_issue_with_index_out_of_range() {
        let mut edit_use_case = given_edit_usecase_with::<TestEditor>(
            Board::default().with_4_typical_issues(),
        );

        let result = edit_use_case.execute(4);

        then_edited_board(&edit_use_case)
            .assert_has_original_issues();

        assert!(matches!(edit_use_case.presenter.errors_presented.as_slice(), [DomainError::IndexOutOfRange(4)]));
        assert!(matches!(result, Err(DomainError::IndexOutOfRange(4))))
    }

    #[test]
    fn test_editor_closes_abruptly() {
        let mut edit_use_case = given_edit_usecase_with::<CloseAbruptlyEditor>(
            Board::default().with_4_typical_issues(),
        );

        let result = edit_use_case.execute(3);

        then_edited_board(&edit_use_case)
            .assert_issue_count(4)
            .assert_has_original_issues();

        assert!(matches!(edit_use_case.presenter.errors_presented.as_slice(), [DomainError::EditorError{..}]));
        assert!(matches!(result, Err(DomainError::EditorError{ .. })), "Expected EditorError, got: {:?}", result)
    }

    fn then_edited_board<E: Editor>(sut: &EditUseCase<MemoryIssueStorage, NilPresenter, E>) -> Board<Issue> {
        sut.storage.load()
    }

    fn then_stored_issue_of_the(board: &Board<Issue>) -> Entity<Issue> {
        let issue = board.get(board.find_entity_id_by_index(2).unwrap());
        issue.clone()
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

    fn given_edit_usecase_with<E: Editor + Default>(board: Board<Issue>) -> EditUseCase<MemoryIssueStorage, NilPresenter, E> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        EditUseCase {
            storage,
            ..Default::default()
        }
    }
}
