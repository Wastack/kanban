use validated::Validated;
use validated::Validated::Fail;
use crate::application::domain::error::DomainError;
use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
use crate::application::issue::Stateful;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::State;


#[derive(Default)]
pub(crate) struct MoveUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> MoveUseCase<I, P> {
    pub(crate) fn execute(&mut self, indices: &[usize], state: State) -> Validated<(), DomainError> {
        let mut board = self.storage.load();

        let validated = board.validate_indices(indices);

        if let Fail(errors) = &validated {
            errors.into_iter()
                .for_each(|e| self.presenter.render_error(&e));

            return validated;
        }

        // TODO there is a bug: if first move changes prio, second index might be invalid

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
            .assert_state_is_done();

        then_issue_with_index(1, &move_use_case)
            .assert_state_is_done();

        let stored_board = move_use_case.storage.load();

        stored_board
            .history()
            .assert_contains_1_moving();

        let presented_board = move_use_case.presenter.last_board_rendered.expect("Expected a board to be presented");
        assert_eq!(presented_board, stored_board, "Expected stored and presented board to be equal");
    }

    /// Tests whether the issue goes on the top of the done list, when being moved there.
    #[test]
    fn test_move_done_results_in_prio_top() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![3], State::Done);

        then_issue_with_index(1, &move_use_case)
            .assert_description("Task inserted first")
            .assert_state_is_done();

        then_issue_with_index(2, &move_use_case)
            .assert_description("Task inserted third")
            .assert_state_is_done();

        let stored_board = move_use_case.storage.load();

        stored_board
            .history()
            .assert_consist_of_1_move_with_index_changed();

        let presented_board = move_use_case.presenter.last_board_rendered.expect("Expected a board to be presented");
        assert_eq!(presented_board, stored_board, "Expected stored and presented board to be equal");
    }

    #[test]
    fn test_indices_out_of_range() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let result = move_use_case.execute(&vec![1, 4, 5], State::Done);

        then_moving(&result)
            .assert_failed()
            .assert_has_two_errors();

        move_use_case.storage.load()
            .assert_issue_count(4)
            .assert_has_original_issues();

        assert!(matches!(move_use_case.presenter.errors_presented.as_slice(), [
            DomainError::IndexOutOfRange(4),
            DomainError::IndexOutOfRange(5),
        ]))
    }

    fn given_move_use_case_with(board: Board) -> MoveUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        MoveUseCase {
            storage,
            ..Default::default()
        }
    }

    fn then_issue_with_index(index: usize, sut: &MoveUseCase<MemoryIssueStorage, NilPresenter>) -> Issue {
        let board = sut.storage.load();

        board.get_issue(index).unwrap().clone()
    }

    impl History {
        fn assert_contains_1_moving(&self) -> &Self {
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

        fn assert_consist_of_1_move_with_index_changed(&self) -> &Self {
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
        fn assert_failed(&self) -> &Self {
            assert!(self.result.is_fail(), "Expected deletion to fail");
            self
        }

        fn assert_has_two_errors(&self) -> &Self {
            let Fail(errors) = self.result else { panic!("Expected moving to fail") };
            assert_eq!(errors.len(), 2, "Expected 2 errors");
            assert!(matches!(errors[0], DomainError::IndexOutOfRange(4)));
            assert!(matches!(errors[1], DomainError::IndexOutOfRange(5)));

            self
        }
    }

    impl Issue {
        fn assert_state_is_done(&self) -> &Self {
            assert_eq!(self.state, State::Done, "Expected moved issue to be in done state");
            self
        }

        fn assert_description(&self, description: &str) -> &Self {
            assert_eq!(self.description, Description::from(description));
            self
        }
    }

}
