use internal_macros::{PresenterHolder, StorageHolder};
use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, UndoableHistoryElement};
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::application::usecase::usecase::{with_board_saved_and_presented_multi_error, HasPresenter, HasStorage};

#[derive(Default, PresenterHolder, StorageHolder)]
pub(crate) struct DeleteUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P
}

impl<I: IssueStorage, P: Presenter> DeleteUseCase<I, P> {
    pub(crate) fn execute(&mut self, indices: &[usize]) {
        with_board_saved_and_presented_multi_error(self, |mut board| {
            let ids = board.find_entities_by_indices(indices)?;

            let history_elements = ids.into_iter().map(|id| {
                let original_index = board.position(id);

                board.mark_as_deleted(id);

                DeleteHistoryElement { original_position_in_issues: original_index }
            }).collect();

            board.history.add(UndoableHistoryElement::Delete(
                DeleteHistoryElements {
                    deletions: history_elements,
                },
            ));

            Ok(board)
        })
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use crate::application::Issue;
    use crate::application::issue::Description;
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::{DeleteHistoryElement, UndoableHistoryElement};
    use crate::application::usecase::delete::DeleteUseCase;
    use crate::application::usecase::test_utils::get_stored_and_presented_board;

    #[test]
    fn test_execute_successful_deletion() {
        let mut sut = given_delete_use_case_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        // When second, fourth and first issue are deleted
        let _ = sut.execute(&vec![1, 3, 0]);

        // Then
        let stored_board = get_stored_and_presented_board(&sut);

        check!(stored_board.entity_count() == 1, "Expected only 1 issue in board after deletion");

        let remaining_issue = stored_board.get_with_index(0);
        check!(remaining_issue.description == Description::from("Task inserted second"), "Expected the third task to remain with index 0");

        let deleted_issues = stored_board.get_deleted_entities();
        check!(deleted_issues.len() == 3, "Expected 3 deleted issues in board");

        check!(deleted_issues[0].description == "Task inserted fourth".into());
        check!(deleted_issues[1].description == "Task inserted first".into());
        check!(deleted_issues[2].description == "Task inserted third".into());

        let stored_history = stored_board.history.last()
            .expect("Expected a delete history");

        let_assert!(UndoableHistoryElement::Delete(stored_delete_history_elements) = stored_history, "Expected history element to be a deletion");

        let_assert!([
                DeleteHistoryElement{ original_position_in_issues: 1 },
                DeleteHistoryElement{ original_position_in_issues: 2 }, // decreased because of index shift
                DeleteHistoryElement{ original_position_in_issues: 0 },
            ]  = stored_delete_history_elements.deletions.as_slice());
    }

    #[test]
    fn test_deletion_index_out_of_range() {
        let mut delete_use_case = given_delete_use_case_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        delete_use_case.execute(&vec![1, 4, 5]);

        let cell = delete_use_case.presenter.errors_presented.borrow();
        let errors = cell.as_slice();
        let_assert!([DomainError::IndexOutOfRange(4), DomainError::IndexOutOfRange(5)] = errors);

        delete_use_case.storage.load()
            .assert_issue_count(4)
            .assert_has_original_issues();
    }

    fn given_delete_use_case_with(board: HistorizedBoard<Issue>) -> DeleteUseCase<MemoryIssueStorage, NilPresenter> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        DeleteUseCase {
            storage,
            ..Default::default()
        }
    }
}

