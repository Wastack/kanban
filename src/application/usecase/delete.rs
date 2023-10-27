use validated::Validated::Fail;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct DeleteUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl DeleteUseCase {
    pub(crate) fn execute(&mut self, indices: &[usize]) {
        let mut board = self.storage.load();

        let validated = board.validate_indices(indices);

        if let Fail(errors) = validated {
            errors.into_iter()
                .for_each(|e| self.presenter.render_error(&e));
            return

        }

        // Sort the indices in descending order,
        // so that each removal does not affect the next index.
        let mut sorted_indices = indices.to_owned();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

        for &i in &sorted_indices {
            board.issues.remove(i);
        }

        self.storage.save(&board);
        self.presenter.render_board(&board);
    }
}

#[cfg(test)]
mod tests {
    use crate::application::{Board, Issue};
    use crate::application::issue::{Described, Description};
    use crate::{DeleteUseCase, IssueStorage, State};
    use crate::adapters::presenters::nil_presenter::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::MemoryIssueStorage;

    #[test]
    fn test_execute_successful_deletion() {
        let mut sut = build_delete_usecase(
            board_with_4_issues(),
        );
        sut.execute(&vec![1, 3, 0]);

        then_only_the_third_issue_remains(&sut);
    }

    #[test]
    fn test_execute_validation_failure() {
        let mut sut = build_delete_usecase(
            board_with_4_issues(),
        );
        sut.execute(&vec![1, 4, 5]);

        then_two_errors_were_presented(&sut);
        then_board_did_not_change(&sut);
    }



    fn then_only_the_third_issue_remains(use_case: &DeleteUseCase) {
        let saved_board = use_case.storage.load();
        assert_eq!(saved_board.issues.len(), 1, "Expected to contain only 1 issue after deletion");

        let Ok(remaining_issue) = saved_board.get_issue(0) else { panic!("Expected to have an issue with index 0") };
        assert_eq!(remaining_issue.description(), &Description::from("Third task"), "Expected the third task to remain with index 0")
    }

    fn then_board_did_not_change(u: &DeleteUseCase) {
        assert_eq!(u.storage.load(), board_with_4_issues(), "Expected board not to have changed")
    }

    fn then_two_errors_were_presented(_u: &DeleteUseCase) {
        // TODO
    }

    fn build_delete_usecase(board: Board) -> DeleteUseCase {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        DeleteUseCase {
            storage: Box::new(storage),
            presenter: Box::new(NilPresenter::default()),
        }
    }

    fn board_with_4_issues() -> Board {
        Board {
            issues: vec![
                Issue {
                    description: Description::from("First task"),
                    state: State::Open,
                    time_created: 1698397489,

                },
                Issue {
                    description: Description::from("Second task"),
                    state: State::Review,
                    time_created: 1698397490,
                },
                Issue {
                    description: Description::from("Third task"),
                    state: State::Done,
                    time_created: 1698397491,
                },
                Issue {
                    description: Description::from("Forth task"),
                    state: State::Open,
                    time_created: 1698397492,
                },
            ],
        }
    }
}

