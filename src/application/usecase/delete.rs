use validated::Validated;
use validated::Validated::Fail;
use crate::application::domain::error::{DomainError};
use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, UndoableHistoryElement};
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct DeleteUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P
}

impl<I: IssueStorage, P: Presenter> DeleteUseCase<I, P> {
    pub(crate) fn execute(&mut self, indices: &[usize]) -> Validated<(), DomainError> {
        let mut board = self.storage.load();
        let ids = match board.find_entities_by_indices(indices) {
            Validated::Good(ids) => ids,
            Fail(errors) => {
                for e in &errors {
                    self.presenter.render_error(e);
                }

                return Fail(errors)
            },
        };

        for id in ids {
            board.mark_as_deleted(id);
        }

        board.history_mut().push(UndoableHistoryElement::Delete(
            DeleteHistoryElements {
                deletions: indices.iter().map(|&i|DeleteHistoryElement {
                    original_position_in_issues: i,
                }).collect(),
            }));

        self.presenter.render_board(&board);
        self.storage.save(&board);

        Validated::Good(())
    }
}

#[cfg(test)]
mod tests {
    use validated::Validated;
    use validated::Validated::Fail;
    use crate::application::{Board};
    use crate::application::issue::{Described, Description};
    use crate::{DeleteUseCase, IssueStorage};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::error::DomainError;

    #[test]
    fn test_execute_successful_deletion() {
        let mut sut = given_delete_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        // When second, fourth and first issue are deleted
        sut.execute(&vec![1, 3, 0]);

        // Then
        let stored_board = sut.storage.load();

        stored_board
            .assert_third_issue_is_the_only_one_left()
            .assert_deleted_issues_consists_of_three_deletions();

        let presented_board = sut.presenter.last_board_rendered.expect("Expected a board to be presented");
        assert_eq!(presented_board, stored_board, "Expected stored and presented board to be equal");
    }

    #[test]
    fn test_deletion_index_out_of_range() {
        let mut delete_use_case = given_delete_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let result = delete_use_case.execute(&vec![1, 4, 5]);

        then_deletion(&result)
            .assert_failed()
            .assert_two_errors_indicated_out_of_range();

        let presented_errors = delete_use_case.presenter
            .errors_presented;
        assert!(matches!(presented_errors.as_slice(), [DomainError::IndexOutOfRange(4), DomainError::IndexOutOfRange(5)]),
            "Expected two index out of range errors to be presented");

        delete_use_case.storage.load()
            .assert_issue_count(4)
            .assert_has_original_issues();
    }

    fn then_deletion(result: &Validated<(), DomainError>) -> DeletionResult {
        DeletionResult {
            result,
        }
    }

    struct DeletionResult<'a> {
        result: &'a Validated<(), DomainError>
    }

    impl DeletionResult<'_> {
        fn assert_failed(&self) -> &Self {
            assert!(self.result.is_fail(), "Expected deletion to fail");
            self
        }

        fn assert_two_errors_indicated_out_of_range(&self) -> &Self {
            let Fail(errors) = self.result else { panic!("Expected deletion to fail") };
            assert_eq!(errors.len(), 2, "Expected 2 errors");
            assert!(matches!(errors[0], DomainError::IndexOutOfRange(4)));
            assert!(matches!(errors[1], DomainError::IndexOutOfRange(5)));

            self
        }
    }


    impl Board {
        fn assert_third_issue_is_the_only_one_left(&self) -> &Self {
            assert_eq!(self.issues_count(), 1, "Expected to contain only 1 issue after deletion");

            let remaining_issue = self.get_issue(0).expect("Expected to have an issue with index 0");
            assert_eq!(remaining_issue.description(), &Description::from("Task inserted second"), "Expected the third task to remain with index 0");

            self
        }

        fn assert_deleted_issues_consists_of_three_deletions(&self) -> &Self {
            let deleted_issues = self.get_deleted_issues();
            assert_eq!(deleted_issues.len(), 3, "Expected 3 deleted issues in board");

            assert_eq!(deleted_issues[0].description, "Task inserted fourth".into());
            assert_eq!(deleted_issues[1].description, "Task inserted first".into());
            assert_eq!(deleted_issues[2].description, "Task inserted third".into());

            self
        }
    }

    fn given_delete_use_case_with(board: Board) -> DeleteUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        DeleteUseCase {
            storage,
            ..Default::default()
        }
    }
}

