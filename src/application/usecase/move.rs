use validated::Validated;
use validated::Validated::Fail;
use crate::application::domain::error::DomainError;
use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
use crate::application::issue::Stateful;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::State;


#[derive(Default)]
pub(crate) struct MoveUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl MoveUseCase {
    pub(crate) fn execute(&mut self, indices: &[usize], state: State) -> Validated<(), DomainError> {
        let mut board = self.storage.load();

        let validated = board.validate_indices(indices);

        if let Fail(errors) = &validated {
            errors.into_iter()
                .for_each(|e| self.presenter.render_error(&e));

            return validated;
        }

        // TODO: could handling of history be handled in a more concise way?
        let mut history_elements = Vec::default();

        for &index in indices {
            let original_state = board.get_issue(index).unwrap().state().clone();

            if original_state == state {
                continue;
            }

            board.move_issue(index, state).unwrap();

            // If issue is moved to done, I'd like to see it on the top
            let new_index = if state == State::Done {
                // TODO watch out, this should not be and undoable event
                board.prio_top_in_category(index)
            } else {
                index
            };

            history_elements.push(MoveHistoryElement {
                new_index,
                original_index: index,
                original_state,
            })
        }

        if !history_elements.is_empty() {
            board.history_mut().push(UndoableHistoryElement::Move(MoveHistoryElements{
                moves: history_elements,
            }));
        }

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Validated::Good(())
    }
}

#[cfg(test)]
mod tests {
    use validated::Validated;
    use validated::Validated::Fail;
    use crate::application::{Board, Issue};
    use crate::{IssueStorage, MoveUseCase, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::error::{DomainError};
    use crate::application::domain::history::{History, MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
    use crate::application::issue::Description;

    #[test]
    fn test_successful_add_use_case() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![1, 0], State::Done);

        then_issue_with_index(0, &move_use_case)
            .has_done_state();

        then_issue_with_index(1, &move_use_case)
            .has_done_state();

        then_stored_board(&move_use_case)
            .history()
            .should_contain_1_move();
    }

    /// Tests whether the issue goes on the top of the done list, when being moved there.
    #[test]
    fn test_move_done_results_in_prio_top() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![3], State::Done);

        then_issue_with_index(1, &move_use_case)
            .has_description("Task inserted first")
            .has_done_state();

        then_issue_with_index(2, &move_use_case)
            .has_description("Task inserted third")
            .has_done_state();

        then_stored_board(&move_use_case)
            .history()
            .should_contain_1_move_with_changing_index();
    }

    #[test]
    fn test_indices_out_of_range() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let result = move_use_case.execute(&vec![1, 4, 5], State::Done);

        then_moving(&result)
            .did_fail()
            .did_produce_two_errors();

        then_stored_board(&move_use_case)
            .has_number_of_issues(4)
            .has_the_original_4_issues();
    }

    fn given_move_use_case_with(board: Board) -> MoveUseCase {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        MoveUseCase {
            storage: Box::new(storage),
            presenter: Box::new(NilPresenter::default()),
        }
    }

    fn then_issue_with_index(index: usize, sut: &MoveUseCase) -> Issue {
        let board = sut.storage.load();

        board.get_issue(index).unwrap().clone()
    }

    fn then_stored_board(u: &MoveUseCase) -> Board {
        u.storage.load()
    }

    impl History {
        fn should_contain_1_move(&self) -> &Self {
            assert!(self.len() >= 1, "Expected an entry in history");
            assert_eq!(self.peek().unwrap(), &UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![
                    MoveHistoryElement {
                        original_state: State::Open,
                        original_index: 0,
                        new_index: 0,
                    },
                ]
            }), "Expected a history element with specific content");

            self
        }

        fn should_contain_1_move_with_changing_index(&self) -> &Self {
            assert!(self.len() >= 1, "Expected an entry in history");
            assert_eq!(self.peek().unwrap(), &UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![
                    MoveHistoryElement {
                        original_state: State::Open,
                        original_index: 3,
                        new_index: 1,
                    },
                ]
            }), "Expected a history element with specific content");

            self
        }
    }

    fn then_moving(result: &Validated<(), DomainError>) -> MovingResult {
        MovingResult {
            result,
        }
    }

    struct MovingResult<'a> {
        result: &'a Validated<(), DomainError>
    }

    impl MovingResult<'_> {
        fn did_fail(&self) -> &Self {
            assert!(self.result.is_fail(), "Expected deletion to fail");
            self
        }

        fn did_produce_two_errors(&self) -> &Self {
            let Fail(errors) = self.result else { panic!("Expected moving to fail") };
            assert_eq!(errors.len(), 2, "Expected to produce 2 errors");
            assert_eq!(errors[0].description(), "Index out of range: 4", "Expected specific error message");
            assert_eq!(errors[1].description(), "Index out of range: 5", "Expected specific error message");
            self
        }
    }

    impl Issue {
        fn has_done_state(&self) -> &Self {
            assert_eq!(self.state, State::Done, "Expected moved issue to be in done state");
            self
        }

        fn has_description(&self, description: &str) -> &Self {
            assert_eq!(self.description, Description::from(description));
            self
        }
    }

}
