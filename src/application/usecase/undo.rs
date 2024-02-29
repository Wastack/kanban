use crate::{IssueStorage, Presenter};
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::{UndoableHistoryElement};

#[derive(Default)]
pub(crate) struct UndoUseCase<I, P> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> UndoUseCase<I, P> {
    pub(crate) fn execute(&mut self) -> DomainResult<()> {
        let mut board = self.storage.load();

        let event_to_undo = board
            .last_history()
            .ok_or(DomainError::EmptyHistory)?
            // todo: if history was just an addendum, clone would not be needed, because only the non-historic board part would need to be mutable
            .clone();

        match event_to_undo {
            UndoableHistoryElement::Add => {
                let id = board
                    .find_entity_id_by_index(0)
                    // In this case, we fail because the board was invalid, not because the user specified a wrong range
                    .map_err(|e| DomainError::InvalidBoard(e.to_string()))
                    .inspect_err(|e|self.presenter.render_error(e))?;

                board.remove(id);
            },
            UndoableHistoryElement::Delete(info) => {
                if board.get_deleted_entities().len() < info.deletions.len() {
                    return Err(DomainError::InvalidBoard(format!("has {} deleted issues, and history suggests to restore {} deleted issues",
                                                                 board.get_deleted_entities().len(),
                                                                 info.deletions.len())));
                }

                // The first number is the index that identifies the element to be restored from
                // the list of deleted issues

                // The second number is the original position of the issue before deletion
                let mut indices_to_restore = info.deletions
                    .to_owned()
                    .into_iter()
                    .enumerate()
                    .map(|(index, d)| (info.deletions.len() - index - 1, d.original_position_in_issues) )
                    .collect::<Vec<_>>();

                // Sort it, so that insertions happen at the right place
                indices_to_restore.sort_unstable_by(|a, b| a.1.cmp(&b.1));

                for &(deleted_index, orignial_index) in &indices_to_restore {
                    // remove from deleted
                    let deleted_issues = board.get_deleted_entities_mut();
                    let issue = deleted_issues[deleted_index].clone();

                    // restore
                    board.insert(orignial_index, issue);
                }

                // clear deleted issues
                let deleted_issues = board.get_deleted_entities_mut();
                deleted_issues.drain(0..indices_to_restore.len());
            },
            UndoableHistoryElement::Prio(_) => {
                return Err(DomainError::NotImplemented)
            },
            UndoableHistoryElement::Edit(_) => {
                return Err(DomainError::NotImplemented)
            },
            UndoableHistoryElement::Move(info) => {
                for h in info.moves.iter().rev() {
                    if h.original_index != h.new_index {
                        let issue = board.remove_by_index(h.new_index);
                        board.insert(h.original_index, issue);
                    }

                    let id = board.find_entity_id_by_index(h.original_index).map_err(
                        |e| DomainError::InvalidBoard(e.to_string())
                    )?;

                    let entity = board.get_mut(id);
                    entity.state = h.original_state;
                }
            },
        }

        board.pop_history();

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }
}


#[cfg(test)]
pub(crate) mod tests {
    use assert2::{check, let_assert};
    use crate::application::{HistorizedBoard, Issue};
    use crate::{IssueStorage, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::adapters::time_providers::fake::{DEFAULT_FAKE_TIME};
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
    use crate::application::issue::{Description};
    use crate::application::usecase::undo::UndoUseCase;

    #[test]
    fn test_undo_add() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_an_issue_added_additionally(),
        );

        let result = undo_use_case.execute();
        assert!(matches!(result, Ok(())), "Expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_issue_count(4)
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_delete_one_issue() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_an_issue_deleted(),
        );

        let result = undo_use_case.execute();
        assert!(matches!(result, Ok(())), "expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_issue_count(4)
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_delete_multiple_issue() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_1_0_2_issues_deleted(),
        );

        let result = undo_use_case.execute();

        assert!(matches!(result, Ok(())), "expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_issue_count(4)
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_on_empty_board() {
        let mut undo_use_case = given_undo_usecase_with( HistorizedBoard::default() );
        let result =undo_use_case.execute();

        assert!(matches!(result, Err(DomainError::EmptyHistory)));
    }

    #[test]
    fn test_undo_move_simple() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_1_moved_from_done_to_open()
        );

        let result = undo_use_case.execute();

        assert!(matches!(result, Ok(())), "expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_issue_count(4)
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_move_with_prio_change() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_issue_moved_to_done()
        );

        let result = undo_use_case.execute();

        assert!(matches!(result, Ok(())), "expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_issue_count(4)
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_2_undos_in_sequence() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_an_issue_added_additionally()
                .with_most_priority_issue_moved_to_review(),
        );

        // When undoing move
        let result = undo_use_case.execute();
        assert!(matches!(result, Ok(())), "expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_has_original_issues()
            .has_additional_issue_added_with_state_open()
            .has_the_addition_in_history();

        // When undoing addition
        let result = undo_use_case.execute();
        assert!(matches!(result, Ok(())), "expected undo usecase to succeed");

        then_board_for(&undo_use_case)
            .assert_has_original_issues()
            .has_original_history();
    }

    #[test]
    fn test_undo_delete_inconsistent_board() {
        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::default()
                .with_4_typical_issues()
                .with_inconsistent_delete_history()
        );
        let result = undo_use_case.execute();

        let_assert!(Err(DomainError::InvalidBoard(error_reason)) = result, "Expected InvalidBoard error");
        assert_eq!(error_reason, "has 2 deleted issues, and history suggests to restore 3 deleted issues", "expected specific reason for InvalidBoard error")
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
            time_created: DEFAULT_FAKE_TIME,
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

        let mut undo_use_case = given_undo_usecase_with(
            HistorizedBoard::new(entities, vec![], history ));

        // When
        let result = undo_use_case.execute();

        // Then
        let_assert!(Ok(()) = result);
        let stored_board = undo_use_case.storage.load();

        for (index, expected_description, expected_state) in [
                (0, "I finished this first", State::Done),
                (1, "Lazy to do", State::Open),
                (2, "I'm doing it now, A", State::Open),
                (3, "I'm doing it now, B", State::Open) ] {
            let issue = stored_board.get(stored_board.find_entity_id_by_index(index).unwrap());
            check!(issue.description.as_str() == expected_description);
            check!(issue.state == expected_state);
        }

        check!(stored_board.history == [], "Expected history to have been cleared");
        let_assert!(Some(presented_board) = undo_use_case.presenter.last_board_rendered, "Expected board to have been presented");

        check_boards_are_equal(&stored_board, &presented_board);
    }

    impl HistorizedBoard<Issue> {
        fn with_an_issue_added_additionally(mut self) -> Self {
            self.append_entity(
                Issue{
                    description: Description::from("Additional Issue"),
                    state: State::Open,
                    time_created: 0,
                }
            );
            self.push_to_history(UndoableHistoryElement::Add);

            self
        }
        fn with_an_issue_deleted(mut self) -> Self {
            self.mark_as_deleted(self.find_entity_id_by_index(2).unwrap());
            self.push_to_history(UndoableHistoryElement::Delete(
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

            self.push_to_history(UndoableHistoryElement::Delete(
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

        fn with_1_moved_from_done_to_open(mut self) -> Self {
            self.get_mut(self.find_entity_id_by_index(1).unwrap()).state = State::Open;
            self.push_to_history(UndoableHistoryElement::Move(MoveHistoryElements{
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
            self.get_mut(self.find_entity_id_by_index(0).unwrap()).state = State::Review;

            self.push_to_history(UndoableHistoryElement::Move(MoveHistoryElements{
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
            [1, 0].into_iter().for_each(|i| self.mark_as_deleted(self.find_entity_id_by_index(i).unwrap()));
            self.push_to_history(UndoableHistoryElement::Delete(
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
            self.push_to_history(UndoableHistoryElement::Move(MoveHistoryElements{
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
            check!(self.last_history().is_none(), "Expected history to be empty");
            self
        }

        fn has_additional_issue_added_with_state_open(&self) -> &Self {
            let issue = self.get(self.find_entity_id_by_index(0).expect("Expected to have an issue"));
            assert_eq!(issue.description, Description::from("Additional Issue"), "Expected Additional Issue in first place");
            assert_eq!(issue.state, State::Open, "Expected issue to be in Open state");

            self
        }

        fn has_the_addition_in_history(&self) -> &Self {
            assert_eq!(self.last_history(), Some(&UndoableHistoryElement::Add), "Expected addition to be present in history as last event");

            self
        }

    }

    fn then_board_for(undo: &UndoUseCase<MemoryIssueStorage, NilPresenter>) -> HistorizedBoard<Issue> {
        undo.storage.load()
    }

    fn given_undo_usecase_with(board: HistorizedBoard<Issue>) -> UndoUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        UndoUseCase {
            storage,
            ..Default::default()
        }
    }

}

