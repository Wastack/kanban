use validated::Validated;
use validated::Validated::Fail;
use crate::application::domain::error::DomainError;
use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
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

        let ids = match board.find_entities_by_indices(indices) {
            Validated::Good(ids) => ids,
            Fail(errors) => {
                for e in &errors {
                    self.presenter.render_error(e);
                }

                return Fail(errors);
            }
        };

        // TODO there is a bug: if first move changes prio, second index might be invalid

        // TODO: could handling of history be handled in a more concise way?
        let mut history_elements = Vec::default();

        for id in ids {
            let issue = board.get_mut(id);
            let original_state = issue.state;

            if issue.state == state {
                continue;
            }

            issue.state = state;

            // TODO: it should be done by id directly
            let current_index = board.entities.iter().position(|e| e.id == id).unwrap();
            // If issue is moved to done, I'd like to see it on the top
            let new_index = if state == State::Done {
                board.prio_top_in_category(current_index)
            } else {
                current_index
            };

            history_elements.push(MoveHistoryElement {
                new_index,
                original_index: current_index,
                original_state,
            })
        }

        if !history_elements.is_empty() {
            board.history.push(UndoableHistoryElement::Move(MoveHistoryElements {
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
    use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
    use crate::application::issue::{Description, Entity};

    #[test]
    fn test_successful_move_use_case() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![1, 0], State::Done);

        then_issue_with_index(0, &move_use_case)
            .assert_state_is(State::Done);

        then_issue_with_index(1, &move_use_case)
            .assert_state_is(State::Done);

        let stored_board = move_use_case.storage.load();

        assert_eq!(stored_board.history.last().expect("Expected an entry in history"),
                   &UndoableHistoryElement::Move(MoveHistoryElements {
                       moves: vec![
                           MoveHistoryElement {
                               original_state: State::Open,
                               original_index: 0,
                               new_index: 0,
                           },
                       ]
                   }), "Expected a history element with specific content");

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
            .assert_state_is(State::Done);

        then_issue_with_index(2, &move_use_case)
            .assert_description("Task inserted third")
            .assert_state_is(State::Done);

        let stored_board = move_use_case.storage.load();

        assert_eq!(stored_board.history.last().expect("Expected element in history"),
                   &UndoableHistoryElement::Move(MoveHistoryElements {
                       moves: vec![
                           MoveHistoryElement {
                               original_state: State::Open,
                               original_index: 3,
                               new_index: 1,
                           },
                       ]
                   }), "Expected a history element with specific content");

        let presented_board = move_use_case.presenter.last_board_rendered.expect("Expected a board to be presented");
        assert_eq!(presented_board, stored_board, "Expected stored and presented board to be equal");
    }

    // TODO: undo counterpart
    /// Open
    /// 1. Lazy to do
    /// 2. I'm doing it now, A
    /// 3. I'm doing it now, B
    ///
    /// Done
    /// 0. I finished this first
    ///
    /// > ka move done 3 2
    ///
    /// Open
    /// 3. Lazy to do
    ///
    /// Done
    /// 0. I'm doing it now, B
    /// 1. I'm doing it now, A
    /// 2. I finished this first
    ///
    /// When index `2` moves to done, it becomes index `0` (so that it appears on the top of the list of DONE items).
    /// Watch out, because during the change index `1` becomes to index `2` So you may end up moving 'Lazy to do' to
    /// `DONE`.
    #[test]
    fn test_move_multiple_to_done_with_changing_priorities() {
        // Given
        let mut sut = given_move_use_case_with(
            Board::default()
                .with_issue(Issue { description: Description::from("I'm doing it now, B"), state: State::Open, time_created: 0 })
                .with_issue(Issue { description: Description::from("I'm doing it now, A"), state: State::Open, time_created: 0 })
                .with_issue(Issue { description: Description::from("Lazy to do"), state: State::Open, time_created: 0 })
                .with_issue(Issue { description: Description::from("I finished this first"), state: State::Done, time_created: 0 })
        );

        // When
        sut.execute(&[3, 2], State::Done).expect("Expected move to succeed");

        // Then
        [
            (0, State::Done, "I'm doing it now, B"),
            (1, State::Done, "I'm doing it now, A"),
            (2, State::Done, "I finished this first"),
            (3, State::Open, "Lazy to do"),
        ].into_iter().for_each(|(expected_index, expected_state, expected_description)| {
            then_issue_with_index(expected_index, &sut)
                .assert_description(expected_description)
                .assert_state_is(expected_state);
        });

        let stored_board = sut.storage.load();
        let presented_board = sut.presenter.last_board_rendered.expect("Expected a board to be presented");
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

    fn given_move_use_case_with(board: Board<Issue>) -> MoveUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        MoveUseCase {
            storage,
            ..Default::default()
        }
    }

    fn then_issue_with_index(index: usize, sut: &MoveUseCase<MemoryIssueStorage, NilPresenter>) -> Entity<Issue> {
        let board = sut.storage.load();

        board.get(board.find_entity_id_by_index(index).unwrap()).clone()
    }

    fn then_moving(result: &Validated<(), DomainError>) -> MovingResult {
        MovingResult {
            result,
        }
    }

    struct MovingResult<'a> {
        result: &'a Validated<(), DomainError>,
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
        fn assert_state_is(&self, s: State) -> &Self {
            assert_eq!(self.state, s, "Expected moved issue to be in state: {:?}", s);
            self
        }

        fn assert_description(&self, description: &str) -> &Self {
            assert_eq!(self.description, Description::from(description));
            self
        }
    }
}
