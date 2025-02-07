use internal_macros::{PresenterHolder, StorageHolder};
use uuid::Uuid;
use crate::adapters::storages::IssueStorage;
use crate::application::board::Board;
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::{DueHistoryElement, EditHistoryElement, FlushHistoryElement, PrioHistoryElement, UndoableHistoryElement};
use crate::application::Issue;
use crate::application::domain::historized_board::HistorizedBoard;
use crate::application::issue::{Description, Entity};
use crate::application::ports::presenter::Presenter;
use crate::application::usecase::usecase::{HasStorage, HasPresenter, with_board_saved_and_presented_single_error};

#[derive(Default, PresenterHolder, StorageHolder)]
pub(crate) struct UndoUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> UndoUseCase<I, P> {
    pub(crate) fn execute(&self) {
        with_board_saved_and_presented_single_error(self, |board| {
            let HistorizedBoard {
                mut board,
                mut history,
            } = board;

            let event_to_undo = history.last()
                .ok_or(DomainError::EmptyHistory)?;

            Self::undo_event(&mut board, event_to_undo)?;

            // When successful, we need to remove the history element that has been undone.
            history.pop();

            let historized_board = HistorizedBoard {
                board,
                history,
            };

            Ok(historized_board)
        })
    }

    /// Undoes and event based on the history element. It does not mutate the history.
    fn undo_event(board: &mut Board<Issue>, history: &UndoableHistoryElement) -> DomainResult<()> {
        match history {
            UndoableHistoryElement::Add => {
                let id = board
                    .find_entity_id_by_index(0)
                    // In this case, we fail because the board was invalid, not because the user specified a wrong range
                    .map_err(|e| DomainError::InvalidBoard(e.to_string()))?;

                board.remove(id);
            },
            UndoableHistoryElement::Delete(info) => {
                if board.get_deleted_entities().len() < info.deletions.len() {
                    return Err(DomainError::InvalidBoard(format!("has {} deleted issues in file, and history suggests to restore {} number of entries",
                                                                 board.get_deleted_entities().len(),
                                                                 info.deletions.len())));
                }

                // Drain issue to be restored and take reversed order,
                // so that indices stores in history denote the right place
                let issues_to_restore = board.get_deleted_entities_mut()
                    .drain(0..info.deletions.len())
                    .collect::<Vec<_>>();


                for (issue, history_element) in issues_to_restore.into_iter().zip(info.deletions.iter()
                    // Take reverse, because in undo, we go backwards.
                    // The first issue that needs restoring is the one that was deleted last
                    .rev()) {
                    // restore
                    board.try_insert(history_element.original_position_in_issues, issue)
                        .map_err(|e| DomainError::InvalidBoard(e.to_string()))?;
                }
            },
            UndoableHistoryElement::Prio(PrioHistoryElement{
                                             original_index,
                                             new_index
                                         }) => {
                let id = Self::try_get_id_or_invalid_board(board, *new_index)?;

                let entity = board.remove(id);
                board.try_insert(*original_index, entity)
                    .map_err(|e| DomainError::InvalidBoard(format!("Original index is out of range: {}", e )))?;
            },
            UndoableHistoryElement::Edit(EditHistoryElement {
                                             original_description,
                                             index }) => {
                let id = Self::try_get_id_or_invalid_board(board, *index)?;
                let issue = board.get_mut(id);

                issue.description = Description::from(original_description.as_str());
            },
            UndoableHistoryElement::Move(info) => {
                for h in info.moves.iter().rev() {
                    if h.original_index != h.new_index {
                        let moved_issue_id = Self::try_get_id_or_invalid_board(board, h.new_index)?;

                        let issue = board.remove(moved_issue_id);
                        board.try_insert(h.original_index, issue)
                            .map_err(|e| DomainError::InvalidBoard(e.to_string()))?;
                    }

                    let id = Self::try_get_id_or_invalid_board(board, h.original_index)?;

                    let entity = board.get_mut(id);
                    entity.state = h.original_state;
                }
            },
            UndoableHistoryElement::Flush(
                FlushHistoryElement{
                    number_of_issues_affected
                }
            ) => {
                if board.get_deleted_entities().len() < *number_of_issues_affected {
                    return Err(DomainError::InvalidBoard(format!("unable to undo flush of {} number of issues, when the total number of issues in deleted entities is {}",
                            number_of_issues_affected,
                            board.get_deleted_entities().len()
                    )))
                }

                let elements_to_restore = {
                    let deleted_entities = board.get_deleted_entities_mut();
                    deleted_entities.drain(..*number_of_issues_affected).collect::<Vec<Entity<Issue>>>()
                };

                for e in elements_to_restore.into_iter() {
                    board.insert(0, e);
                }
            },
            UndoableHistoryElement::Due(
              DueHistoryElement{
                  index, previous_due
              }
            ) => {
                let id = Self::try_get_id_or_invalid_board(board, *index)?;
                let issue = board.get_mut(id);

                issue.due_date = previous_due.clone();
            }
        };

        Ok(())
    }

    fn try_get_id_or_invalid_board(board: &mut Board<Issue>, index: usize) -> Result<Uuid, DomainError> {
        let id = board.find_entity_id_by_index(index)
            .map_err(|e| DomainError::InvalidBoard(format!("Index is out of range: {}", e)))?;
        Ok(id)
    }
}


#[cfg(test)]
pub(crate) mod tests {
    use assert2::{check, let_assert};
    use crate::application::{Issue, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::adapters::time_providers::fake::{DEFAULT_FAKE_TODAY};
    use crate::application::domain::error::DomainError;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, EditHistoryElement, FlushHistoryElement, History, MoveHistoryElement, MoveHistoryElements, PrioHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::test_utils::{check_no_errors, get_stored_and_presented_board};
    use crate::application::usecase::undo::UndoUseCase;
    use crate::application::usecase::usecase::HasPresenter;

    #[test]
    fn test_undo_add() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_an_issue_added_additionally(),
        );

        undo_use_case.execute();

        let stored_board = get_stored_and_presented_board(&undo_use_case);
        check!(stored_board.entity_count() == 4);
        stored_board
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_delete_one_issue() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_an_issue_deleted(),
        );

        undo_use_case.execute();

        let stored_board = get_stored_and_presented_board(&undo_use_case);
        check!(stored_board.entity_count() == 4);
        stored_board
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_delete_multiple_issue() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_1_0_2_issues_deleted(),
        );

        undo_use_case.execute();

        let stored_board = get_stored_and_presented_board(&undo_use_case);
        check!(stored_board.entity_count() == 4);

        stored_board
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_on_empty_board() {
        let undo_use_case = given_undo_usecase_with( HistorizedBoard::default() );

        undo_use_case.execute();

        let cell = undo_use_case.presenter.errors_presented.borrow();
        let maybe_error = cell.last();
        let_assert!(Some(DomainError::EmptyHistory) = maybe_error);
    }

    #[test]
    fn test_undo_move_simple() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_1_moved_from_done_to_open()
        );

        undo_use_case.execute();

        let stored_board = get_stored_and_presented_board(&undo_use_case);
        check!(stored_board.entity_count() == 4);

        stored_board
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_move_with_prio_change() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_issue_moved_to_done()
        );

        undo_use_case.execute();

        let stored_board = get_stored_and_presented_board(&undo_use_case);
        check!(stored_board.entity_count() == 4);

        stored_board
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_2_undos_in_sequence() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_an_issue_added_additionally()
                .with_most_priority_issue_moved_to_review(),
        );

        // When undoing move
        undo_use_case.execute();

        let stored_board = get_stored_and_presented_board(&undo_use_case);
        check!(stored_board.entity_count() == 5);

        stored_board
            .assert_has_original_issues()
            .has_additional_issue_added_with_state_open()
            .has_the_addition_in_history();

        // When undoing addition
        undo_use_case.execute();

        then_board_for(&undo_use_case)
            .assert_has_original_issues()
            .has_original_history();
    }

    #[test]
    fn test_undo_delete_inconsistent_board() {
        // Given
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_inconsistent_delete_history()
        );

        // When
        undo_use_case.execute();

        // Then
        let cell = undo_use_case.presenter.errors_presented.borrow();
        let error = cell.last()
            .expect("Expected error to have been presented");

        let_assert!(DomainError::InvalidBoard(error_reason) = error, "Expected InvalidBoard error");
        assert_eq!(error_reason, "has 2 deleted issues in file, and history suggests to restore 3 number of entries", "expected specific reason for InvalidBoard error")
    }

    #[test]
    fn test_undo_empty_history() {
        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
        );
        undo_use_case.execute();

        // Then
        let cell = undo_use_case.presenter_ref().errors_presented.borrow();
        let error = cell.last()
            .expect("Expected error to have been presented");

        let_assert!(DomainError::EmptyHistory = error, "Expected Empty History error");
    }

    #[test]
    fn test_undo_invalid_add() {
        // Given
        let board = HistorizedBoard::new( vec![], vec![], vec![UndoableHistoryElement::Add]);
        let undo_use_case = given_undo_usecase_with(board);

        // When
        undo_use_case.execute();

        // Then
        let cell = undo_use_case.presenter.errors_presented.borrow();
        let err = cell.last().expect("Expected error");
        let_assert!(DomainError::InvalidBoard(error_message) = err);
        check!(error_message.as_str() == "Index `0` is out of range");
    }

    #[test]
    fn test_undo_move_invalid_original_index() {
        // Given
        let board = HistorizedBoard::new( vec![
            Issue {
                description: Description::from("One task"),
                state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None
            }
        ], vec![], vec![UndoableHistoryElement::Move(MoveHistoryElements{
            moves: vec![MoveHistoryElement{
                original_index: 1, // History suggests a non-existent second task was moved
                original_state: State::Review,
                new_index: 1,
            }],
        })]);

        let undo_use_case = given_undo_usecase_with(board);

        // When
        undo_use_case.execute();

        // then
        let cell = undo_use_case.presenter.errors_presented.borrow();
        let err = cell.last().expect("Expected error");
        let_assert!(DomainError::InvalidBoard(error_message) = err);
        check!(error_message.as_str() == "Index is out of range: Index `1` is out of range");

    }

    /// Testing undoing a command of complicated moves, where multiple issues are moved to done,
    /// which causes priority changes.
    #[test]
    fn test_multi_move_with_prio_change_undo() {
        // Given
        let entities = [
            (State::Done, "I'm doing it now, A"),
            (State::Done, "I'm doing it now, B"),
            (State::Done, "I finished this first"),
            (State::Open, "Lazy to do"),
        ].into_iter().map(|(state, description)| Issue {
            description: Description::from(description),
            state,
            time_created: DEFAULT_FAKE_TODAY,
            due_date: None,
        }).collect();

        let history = vec![UndoableHistoryElement::Move(MoveHistoryElements {
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
        })];

        let undo_use_case = given_undo_usecase_with(
            HistorizedBoard::new(entities, vec![], history ));

        // When
        undo_use_case.execute();

        check_no_errors(&undo_use_case);

        // Then
        let stored_board = get_stored_and_presented_board(&undo_use_case);

        for (index, expected_description, expected_state) in [
                (0, "I finished this first", State::Done),
                (1, "Lazy to do", State::Open),
                (2, "I'm doing it now, A", State::Open),
                (3, "I'm doing it now, B", State::Open) ] {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(index).unwrap());
            check!(issue.description.as_str() == expected_description);
            check!(issue.state == expected_state);
        }

        check!(stored_board.history == History::default(), "Expected history to have been cleared");
    }

    #[test]
    fn test_undo_priority_upwards() {
        // given:
        let board = HistorizedBoard::new(given_swapped_entities(), vec![], vec![
            UndoableHistoryElement::Prio(PrioHistoryElement{
                original_index: 1,
                new_index: 0,
            })
        ]);

        let use_case = given_undo_usecase_with(board);

        // when
        use_case.execute();

        // then
        check_no_errors(&use_case);
        let stored_board = get_stored_and_presented_board(&use_case);
        check_priorities_unswapped(&stored_board);
    }

    #[test]
    fn test_undo_priority_downwards() {
        // given:
        let board = HistorizedBoard::new(given_swapped_entities(), vec![], vec![
            UndoableHistoryElement::Prio(PrioHistoryElement{
                original_index: 0,
                new_index: 1,
            })
        ]);

        let use_case = given_undo_usecase_with(board);

        // when
        use_case.execute();

        // then
        check_no_errors(&use_case);
        let stored_board = get_stored_and_presented_board(&use_case);
        check_priorities_unswapped(&stored_board);
        let_assert!(None = stored_board.history.last(), "Expected history element to be removed after undo");
    }

    #[test]
    fn test_undo_priority_invalid_original_index() {
        // given
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![
            Issue { description: Description::from("An issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }
        ], vec![], vec![
            UndoableHistoryElement::Prio(PrioHistoryElement{ original_index: 1, new_index: 0 })
        ]));

        // when
        use_case.execute();

        let errors_presented = use_case.presenter.errors_presented.borrow();
        let_assert!([DomainError::InvalidBoard(error_message)] = errors_presented.as_slice(), "Expected an error to have occurred");
        check!(error_message == "Original index is out of range: Index `1` is out of range");
    }

    #[test]
    fn test_undo_prority_invalid_new_index() {
        // given
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![
            Issue { description: Description::from("An issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }
        ], vec![], vec![
            UndoableHistoryElement::Prio(PrioHistoryElement{ original_index: 0, new_index: 1 })
        ]));

        // when
        use_case.execute();

        let errors_presented = use_case.presenter.errors_presented.borrow();
        let_assert!([DomainError::InvalidBoard(error_message)] = errors_presented.as_slice(), "Expected an error to have occurred");
        check!(error_message == "Index is out of range: Index `1` is out of range");
    }

    #[test]
    fn test_undo_delete_invalid_original_index() {
        // given
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![
            Issue { description: Description::from("An issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }
        ], vec![
            Issue { description: Description::from("A deleted issue"), state: State::Review,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }
        ], vec![
            UndoableHistoryElement::Delete(DeleteHistoryElements{
                deletions: vec![DeleteHistoryElement{ original_position_in_issues: 2 }],
            })
        ]));

        // when
        use_case.execute();

        let errors_presented = use_case.presenter.errors_presented.borrow();
        let_assert!([DomainError::InvalidBoard(error_message)] = errors_presented.as_slice(), "Expected an error to have occurred");
        check!(error_message == "Index `2` is out of range");
    }

    #[test]
    fn test_undo_move_invalid_new_index() {
        // given
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![
            Issue { description: Description::from("An issue"), state: State::Done,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }
        ], vec![], vec![
            UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![MoveHistoryElement{
                    original_index: 0,
                    original_state: State::Open,
                    new_index: 123,
                }],
            })
        ]));

        // when
        use_case.execute();

        let errors_presented = use_case.presenter.errors_presented.borrow();
        let_assert!([DomainError::InvalidBoard(error_message)] = errors_presented.as_slice(), "Expected an error to have occurred");
        check!(error_message == "Index is out of range: Index `123` is out of range");
    }

    #[test]
    fn test_undo_flush_not_enough_deleted_items() {
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![], vec![
            Issue { description: Description::from("First deleted issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            },
            Issue { description: Description::from("Second deleted issue"), state: State::Review,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            },
        ], vec![
            UndoableHistoryElement::Flush(FlushHistoryElement{
                number_of_issues_affected: 3,
            })
        ]));

        // when
        use_case.execute();

        let errors_presented = use_case.presenter.errors_presented.borrow();
        let_assert!([DomainError::InvalidBoard(error_message)] = errors_presented.as_slice());
        check!(error_message == "unable to undo flush of 3 number of issues, when the total number of issues in deleted entities is 2");
    }

    #[test]
    fn test_undo_flush() {
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![
            Issue { description: Description::from("An issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }
        ], vec![
            Issue { description: Description::from("First deleted issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            },
            Issue { description: Description::from("Second deleted issue"), state: State::Review,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            },
            Issue { description: Description::from("Third deleted issue"), state: State::Open,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            },
            Issue { description: Description::from("Fourth deleted issue"), state: State::Done,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            },
        ], vec![
            UndoableHistoryElement::Flush(FlushHistoryElement{
                number_of_issues_affected: 3,
            })
        ]));

        // when
        use_case.execute();

        check_no_errors(&use_case);
        let stored_board = get_stored_and_presented_board(&use_case);

        check!(stored_board.entity_count() == 3 + 1);
        for (expected_description, actual_entity) in [
            "Third deleted issue",
            "Second deleted issue",
            "First deleted issue",
            "An issue"].into_iter().zip(stored_board.entities()) {
            check!(actual_entity.description == Description::from(expected_description));
        }
    }

    #[test]
    fn test_undo_edit() {
        let use_case = given_undo_usecase_with(HistorizedBoard::new(vec![
            Issue { description: Description::from("An edited issue"), state: State::Open, time_created: DEFAULT_FAKE_TODAY, due_date: None }
        ], vec![], vec![
            UndoableHistoryElement::Edit(EditHistoryElement{
                original_description: String::from("An issue"),
                index: 0,
            })
        ]));

        // when
        use_case.execute();

        let stored_board = get_stored_and_presented_board(&use_case);

        let description = &stored_board.entities().first().expect("Expected entity to be present").description;
        check!(description == &Description::from("An issue"));

        check!(stored_board.history.stack == []);
    }

    fn check_priorities_unswapped(stored_board: &HistorizedBoard<Issue>) {
        for (index, expected_description) in [(0, "This was originally first"), (1, "This was originally second")] {
            let actual_description = stored_board.get(stored_board.find_entity_id_by_index(index).expect("entity to exist")).description.as_str();
            check!(expected_description == actual_description);
        }
    }

    fn given_swapped_entities() -> Vec<Issue> {
        ["This was originally second", "This was originally first"].map(|d| Issue {
            description: Description::from(d),
            state: State::Open,
            time_created: DEFAULT_FAKE_TODAY,
            due_date: None,
        }).to_vec()
    }

    impl HistorizedBoard<Issue> {
        fn with_an_issue_added_additionally(mut self) -> Self {
            self.append_entity(
                Issue{
                    description: Description::from("Additional Issue"),
                    state: State::Open,
                    time_created: DEFAULT_FAKE_TODAY,
                    due_date: None,
                }
            );
            self.history.add(UndoableHistoryElement::Add);

            self
        }
        fn with_an_issue_deleted(mut self) -> Self {
            let id = self.find_entity_id_by_index(2).unwrap();
            self.mark_as_deleted(id);
            self.history.add(UndoableHistoryElement::Delete(
                DeleteHistoryElements {
                    deletions: vec![
                        DeleteHistoryElement{
                            original_position_in_issues: 2,
                        },
                    ]
                }));

            self

        }

        fn with_1_0_2_issues_deleted(mut self) -> Self {
            self.find_entities_by_indices(&[1, 0, 2])
                .unwrap()
                .into_iter()
                .for_each(|id| self.mark_as_deleted(id));

            self.history.add(UndoableHistoryElement::Delete(
                DeleteHistoryElements {
                    deletions: vec![
                        DeleteHistoryElement{
                            original_position_in_issues: 1,
                        },
                        DeleteHistoryElement{
                            original_position_in_issues: 0,
                        },
                        // This would have been the second one, but 1, 0 already disappeared
                        DeleteHistoryElement{
                            original_position_in_issues: 0,
                        },
                    ]
                }));

            self
        }

        fn with_1_moved_from_done_to_open(mut self) -> Self {
            let id = self.find_entity_id_by_index(1).unwrap();
            self.get_mut(id).state = State::Open;
            self.history.add(UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![
                    MoveHistoryElement {
                        new_index: 1,
                        original_index: 1,
                        original_state: State::Done,
                    }
                ]
            }));

            self
        }

        fn with_most_priority_issue_moved_to_review(mut self) -> Self {
            let id = self.find_entity_id_by_index(0);
            self.get_mut(id.unwrap()).state = State::Review;

            self.history.add(UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![
                    MoveHistoryElement {
                        original_state: State::Open,
                        new_index: 0,
                        original_index: 0,
                    }
                ]
            }));

            self
        }

        fn with_inconsistent_delete_history(mut self) -> Self {
            // There is one less issue actually deleted compared to what history suggests
            [1, 0].into_iter().for_each(|i| {
                let id = self.find_entity_id_by_index(i).unwrap();
                self.mark_as_deleted(id)
            });
            self.history.add(UndoableHistoryElement::Delete(
                DeleteHistoryElements {
                    deletions: vec![
                        DeleteHistoryElement{
                            original_position_in_issues: 1,
                        },
                        DeleteHistoryElement{
                            original_position_in_issues: 0,
                        },
                        DeleteHistoryElement{
                            original_position_in_issues: 2,
                        },
                    ]
                }));

            self
        }

        fn with_issue_moved_to_done(mut self) -> Self {
            let id = self.find_entity_id_by_index(2).unwrap();
            self.get_mut(id).state = State::Done;
            self.prio_top_in_category(id);
            self.history.add(UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![
                    MoveHistoryElement{
                        original_index: 2,
                        new_index: 1,
                        original_state: State::Review,
                    }
                ]
            }));

            self
        }

        fn has_original_history(&self) -> &Self {
            check!(self.history.last().is_none(), "Expected history to be empty");
            self
        }

        fn has_additional_issue_added_with_state_open(&self) -> &Self {
            let issue = self.get(self.find_entity_id_by_index(0).expect("Expected to have an issue"));
            assert_eq!(issue.description, Description::from("Additional Issue"), "Expected Additional Issue in first place");
            assert_eq!(issue.state, State::Open, "Expected issue to be in Open state");

            self
        }

        fn has_the_addition_in_history(&self) -> &Self {
            assert_eq!(self.history.last(), Some(&UndoableHistoryElement::Add), "Expected addition to be present in history as last event");

            self
        }

    }

    fn then_board_for(undo: &UndoUseCase<MemoryIssueStorage, NilPresenter>) -> HistorizedBoard<Issue> {
        undo.storage.load()
    }

    fn given_undo_usecase_with(board: HistorizedBoard<Issue>) -> UndoUseCase<MemoryIssueStorage, NilPresenter> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        UndoUseCase {
            storage,
            ..Default::default()
        }
    }

}

