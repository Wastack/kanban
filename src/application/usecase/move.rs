use internal_macros::{PresenterHolder, StorageHolder};
use uuid::Uuid;
use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
use crate::application::Issue;
use crate::application::domain::historized_board::HistorizedBoard;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::application::State;
use crate::application::usecase::usecase::{HasStorage, HasPresenter, with_board_saved_and_presented_multi_error};


#[derive(Default, StorageHolder, PresenterHolder)]
pub(crate) struct MoveUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> MoveUseCase<I, P> {
    pub(crate) fn execute(&mut self, indices: &[usize], state: State) {
        with_board_saved_and_presented_multi_error(self, |mut board| {
            let ids = board.find_entities_by_indices(indices)?;

            let history_for_undo = ids.into_iter()
                .map(|id| Self::move_issue(&mut board, id, state))
                .flatten()
                .collect();

            Self::update_history(&mut board, history_for_undo);

            Ok(board)
        })
    }

    fn update_history(board: &mut HistorizedBoard<Issue>, history_elements: Vec<MoveHistoryElement>) {
        if !history_elements.is_empty() {
            board.history.add(UndoableHistoryElement::Move(MoveHistoryElements {
                moves: history_elements,
            }));
        }
    }

    fn move_issue(board: &mut HistorizedBoard<Issue>, id: Uuid, state: State) -> Option<MoveHistoryElement> {
        let issue = board.get_mut(id);

        if issue.state == state {
            return None
        }

        let original_state = issue.state;
        issue.state = state;

        let original_index = board.position(id);

        board.prio_top_in_category(id);

        // If issue is moved to done, I'd like to see it on the top
        let new_index = if state == State::Done {
            board.position(id)
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
    use crate::application::{Issue, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::adapters::time_providers::fake::DEFAULT_FAKE_TODAY;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::{MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::r#move::MoveUseCase;
    use crate::application::usecase::test_utils::get_stored_and_presented_board;

    #[test]
    fn test_successful_move_use_case() {
        let mut move_use_case = given_move_use_case_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![1, 0], State::Done);

        let stored_board = get_stored_and_presented_board(&move_use_case);
        for index in 0..1 {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(index).unwrap());
            check!(issue.state == State::Done);

        }

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
    }

    /// Tests whether the issue goes on the top of the done list, when being moved there.
    #[test]
    fn test_move_done_results_in_prio_top() {
        let mut move_use_case = given_move_use_case_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![3], State::Done);

        let stored_board = get_stored_and_presented_board(&move_use_case);

        for (index, expected_description) in [(1, "Task inserted first"), (2, "Task inserted third")] {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(index).unwrap());
            check!(issue.description.as_str() == expected_description);
            check!(issue.state == State::Done);

        }

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

    }

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
            HistorizedBoard::new(vec![
                Issue { description: Description::from("I finished this first"), state: State::Done,
                    time_created: DEFAULT_FAKE_TODAY,
                    due_date: None,
                },
                Issue { description: Description::from("Lazy to do"), state: State::Open,
                    time_created: DEFAULT_FAKE_TODAY,
                    due_date: None,
                },
                Issue { description: Description::from("I'm doing it now, A"), state: State::Open,
                    time_created: DEFAULT_FAKE_TODAY,
                    due_date: None,
                }, // Move this second
                Issue { description: Description::from("I'm doing it now, B"), state: State::Open,
                    time_created: DEFAULT_FAKE_TODAY,
                    due_date: None,
                }, // Move this first
            ], vec![], vec![])
        );

        // When
        sut.execute(&[3, 2], State::Done);

        // Then
        let stored_board = get_stored_and_presented_board(&sut);
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

        assert_eq!(stored_board.history.last().expect("Expected element in history"),
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
    }


    #[test]
    fn test_indices_out_of_range() {
        let mut move_use_case = given_move_use_case_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![1, 4, 5], State::Done);

        let errors = move_use_case.presenter.errors_presented.borrow();
        let_assert!([DomainError::IndexOutOfRange(4), DomainError::IndexOutOfRange(5)] = errors.as_slice());

        let stored_board = move_use_case.storage.load();
        stored_board
            .assert_issue_count(4)
            .assert_has_original_issues();
    }

    fn given_move_use_case_with(board: HistorizedBoard<Issue>) -> MoveUseCase<MemoryIssueStorage, NilPresenter> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        MoveUseCase {
            storage,
            ..Default::default()
        }
    }
}
