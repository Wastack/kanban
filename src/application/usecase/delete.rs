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
    use crate::{DeleteUseCase, State};
    use crate::adapters::presenters::nil_presenter::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::MemoryIssueStorage;

    #[test]
    fn test_execute_successful_deletion() {
        let mut use_case = build_test_delete_usecase();
        given_board_with_4_issues_stored(&mut use_case);

        // When executing use-case
        use_case.execute(&vec![1, 3, 0]);

        // TODO is presenter called?

        let saved_board = use_case.storage.load();
        assert_eq!(saved_board.issues.len(), 1, "Expected to contain only 1 issue after deletion");

        let Ok(remaining_issue) = saved_board.get_issue(0) else { panic!("Expected to have an issue with index 0") };
        assert_eq!(remaining_issue.description(), &Description::from("Third task"), "Expected the third task to remain with index 0")
    }

    fn build_test_delete_usecase() -> DeleteUseCase {
        DeleteUseCase {
            storage: Box::new(MemoryIssueStorage::default()),
            presenter: Box::new(NilPresenter::default()),
        }
    }

    #[test]
    fn test_execute_validation_failure() {
    }

    fn given_board_with_4_issues_stored(u: &mut DeleteUseCase) {
        let board = Board {
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
        };

        u.storage.save(&board);
    }
}

