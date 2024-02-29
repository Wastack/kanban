use uuid::Uuid;
use crate::application::domain::error::{DomainResultMultiError};
use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
use crate::application::{Board, Issue};
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::State;


#[derive(Default)]
pub(crate) struct MoveUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> MoveUseCase<I, P> {
    pub(crate) fn execute(&mut self, indices: &[usize], state: State) -> DomainResultMultiError<()> {
        let mut board = self.storage.load();

        let ids = board.find_entities_by_indices(indices)
            .inspect_err(|errors| self.presenter.render_errors(errors))?;

        let history_for_undo = ids.into_iter()
            .map(|id| Self::move_issue(&mut board, id, state))
            .flatten()
            .collect();

        Self::update_history(&mut board, history_for_undo);

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }

    fn update_history(board: &mut Board<Issue>, history_elements: Vec<MoveHistoryElement>) {
        if !history_elements.is_empty() {
            board.push_to_history(UndoableHistoryElement::Move(MoveHistoryElements {
                moves: history_elements,
            }));
        }
    }

    fn move_issue(board: &mut Board<Issue>, id: Uuid, state: State) -> Option<MoveHistoryElement> {
        let issue = board.get_mut(id);

        if issue.state == state {
            return None
        }

        let original_state = issue.state;
        issue.state = state;

        let original_index = board.position(id);

        // If issue is moved to done, I'd like to see it on the top
        let new_index = if state == State::Done {
            board.prio_top_in_category(id)
        } else {
            original_index
        };

        Some(MoveHistoryElement {
            original_state,
            original_index,
            new_index
        })
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use crate::application::{Board, Issue};
    use crate::{IssueStorage, MoveUseCase, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::error::{DomainError, DomainResultMultiError};
    use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
    use crate::application::issue::{Description};

    #[test]
    fn test_successful_move_use_case() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let _ = move_use_case.execute(&vec![1, 0], State::Done);

        let stored_board = move_use_case.storage.load();
        for index in 0..1 {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(index).unwrap());
            check!(issue.state == State::Done);

        }

        assert_eq!(stored_board.last_history().expect("Expected an entry in history"),
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
        check_boards_are_equal(&presented_board, &stored_board);
    }

    /// Tests whether the issue goes on the top of the done list, when being moved there.
    #[test]
    fn test_move_done_results_in_prio_top() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let _ = move_use_case.execute(&vec![3], State::Done);

        let stored_board = move_use_case.storage.load();

        for (index, expected_description) in [(1, "Task inserted first"), (2, "Task inserted third")] {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(index).unwrap());
            check!(issue.description.as_str() == expected_description);
            check!(issue.state == State::Done);

        }

        assert_eq!(stored_board.last_history().expect("Expected element in history"),
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
        check_boards_are_equal(&presented_board, &stored_board);
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
            Board::new(vec![
                Issue { description: Description::from("I finished this first"), state: State::Done, time_created: 0 },
                Issue { description: Description::from("Lazy to do"), state: State::Open, time_created: 0 },
                Issue { description: Description::from("I'm doing it now, A"), state: State::Open, time_created: 0 }, // Move this second
                Issue { description: Description::from("I'm doing it now, B"), state: State::Open, time_created: 0 }, // Move this first
            ], vec![], vec![])
        );

        // When
        sut.execute(&[3, 2], State::Done).expect("Expected move to succeed");

        // Then
        let stored_board = sut.storage.load();
        [
            (0, State::Done, "I'm doing it now, A"),
            (1, State::Done, "I'm doing it now, B"),
            (2, State::Done, "I finished this first"),
            (3, State::Open, "Lazy to do"),
        ].into_iter().for_each(|(expected_index, expected_state, expected_description)| {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(expected_index).unwrap());
            check!(issue.state == expected_state);
            check!(issue.description.as_str() == expected_description);
        });

        assert_eq!(stored_board.last_history().expect("Expected element in history"),
                   &UndoableHistoryElement::Move(MoveHistoryElements {
                       moves: vec![
                           MoveHistoryElement {
                               original_state: State::Open,
                               original_index: 3,
                               new_index: 0,
                           },
                           MoveHistoryElement {
                               original_state: State::Open,
                               original_index: 3,
                               new_index: 0,
                           },
                       ]
                   }), "Expected a history element with specific content");

        let presented_board = sut.presenter.last_board_rendered.expect("Expected a board to be presented");
        check_boards_are_equal(&presented_board, &stored_board);
    }


    #[test]
    fn test_indices_out_of_range() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let result = move_use_case.execute(&vec![1, 4, 5], State::Done);

        then_moving(&result)
            .assert_has_two_errors();

        let stored_board = move_use_case.storage.load();
        stored_board
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

    fn then_moving(result: &DomainResultMultiError<()>) -> MovingResult {
        MovingResult {
            result,
        }
    }

    struct MovingResult<'a> {
        result: &'a DomainResultMultiError<()>,
    }

    impl MovingResult<'_> {
        fn assert_has_two_errors(&self) -> &Self {
            let_assert!(Err(errors) = self.result);
            assert_eq!(errors.len(), 2, "Expected 2 errors");
            assert!(matches!(errors[0], DomainError::IndexOutOfRange(4)));
            assert!(matches!(errors[1], DomainError::IndexOutOfRange(5)));

            self
        }
    }

}
