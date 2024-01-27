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

        let history = board.history();

        let event_to_undo = history
            .peek()
            .ok_or(DomainError::EmptyHistory)?;

        match event_to_undo {
            UndoableHistoryElement::Add => {
                // TODO too much intimacy with the field
                // Board delete method puts field to deleted, which is not correct in this case

                let issues = board.issues_mut();
                issues.remove(0);
            },
            UndoableHistoryElement::Delete(info) => {
                if board.get_deleted_issues().len() < info.deletions.len() {
                    return Err(DomainError::InvalidBoard(format!("has {} deleted issues, and history suggests to restore {} deleted issues",
                                                                 board.get_deleted_issues().len(),
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
                    let deleted_issues = board.get_deleted_issues_mut();
                    let issue = deleted_issues[deleted_index].clone();

                    // restore
                    let issues = board.issues_mut();
                    issues.insert(orignial_index, issue);
                }

                // clear deleted issues
                let deleted_issues = board.get_deleted_issues_mut();
                deleted_issues.drain(0..indices_to_restore.len());
            },
            UndoableHistoryElement::Prio(_) => {
                return Err(DomainError::NotImplemented)
            },
            UndoableHistoryElement::Edit(_) => {
                return Err(DomainError::NotImplemented)
            },
            UndoableHistoryElement::Move(info) => {
                // TODO let us not clone history
                let info = info.clone();
                for h in info.moves.iter().rev() {
                    if h.original_index != h.new_index {
                        // TODO too much intimicy?
                        let issues = board.issues_mut();
                        let issue = issues.remove(h.new_index);
                        issues.insert(h.original_index, issue);
                    }

                    let result = board.move_issue(h.original_index, h.original_state);
                    if let Err(err) = result {
                        return Err(DomainError::InvalidBoard(err.to_string()));
                    }
                }
            },
        }

        board.history_mut().pop();

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }
}


#[cfg(test)]
pub(crate) mod tests {
    use crate::application::{Board, Issue};
    use crate::{IssueStorage, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, MoveHistoryElement, MoveHistoryElements, UndoableHistoryElement};
    use crate::application::issue::{Described, Description};
    use crate::application::usecase::undo::UndoUseCase;

    #[test]
    fn test_undo_add() {
        let mut undo_use_case = given_undo_usecase_with(
            Board::default()
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
            Board::default()
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
            Board::default()
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
        let mut undo_use_case = given_undo_usecase_with( Board::default() );
        let result =undo_use_case.execute();

        assert!(matches!(result, Err(DomainError::EmptyHistory)));
    }

    #[test]
    fn test_undo_move_simple() {
        let mut undo_use_case = given_undo_usecase_with(
            Board::default()
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
            Board::default()
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
            Board::default()
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
            Board::default()
                .with_4_typical_issues()
                .with_inconsistent_delete_history()
        );
        let result = undo_use_case.execute();

        let Err(DomainError::InvalidBoard(error_reason)) = result else { panic!("Expected InvalidBoard error") };
        assert_eq!(error_reason, "has 2 deleted issues, and history suggests to restore 3 deleted issues", "expected specific reason for InvalidBoard error")

    }


    /*
        Test implementation comes here
    */


    impl Board {
        fn with_an_issue_added_additionally(mut self) -> Self {
            self.add_issue(
                Issue::new( Description::from("Additional Issue"), State::Open)
            );
            self.history_mut().push(UndoableHistoryElement::Add);

            self
        }
        fn with_an_issue_deleted(mut self) -> Self {
            self.delete_issues_with(&[2]);
            self.history_mut().push(UndoableHistoryElement::Delete(
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
            self.delete_issues_with(&[1, 0, 2]);
            self.history_mut().push(UndoableHistoryElement::Delete(
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
            self.move_issue(1, State::Open).unwrap();
            self.history_mut().push(UndoableHistoryElement::Move(MoveHistoryElements{
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
            self.move_issue(0, State::Review).unwrap();

            self.history_mut().push(UndoableHistoryElement::Move(MoveHistoryElements{
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
            self.delete_issues_with(&[1, 0]);
            self.history_mut().push(UndoableHistoryElement::Delete(
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
            // TODO too much logic in test
            self.move_issue(2, State::Done).unwrap();
            self.prio_top_in_category(2);
            self.history_mut().push(UndoableHistoryElement::Move(MoveHistoryElements{
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
            assert!(self.history().peek().is_none(), "Expected history to be empty, got: {:?}", self.history());
            self
        }

        fn has_additional_issue_added_with_state_open(&self) -> &Self {
            let issue = self.get_issue(0).expect("Expected to have an issue");
            assert_eq!(issue.description(), &Description::from("Additional Issue"), "Expected Additional Issue in first place");
            assert_eq!(issue.state, State::Open, "Expected issue to be in Open state");

            self
        }

        fn has_the_addition_in_history(&self) -> &Self {
            assert_eq!(self.history().peek(), Some(&UndoableHistoryElement::Add), "Expected addition to be present in history as last event");

            self
        }

    }

    fn then_board_for(undo: &UndoUseCase<MemoryIssueStorage, NilPresenter>) -> Board {
        undo.storage.load()
    }

    fn given_undo_usecase_with(board: Board) -> UndoUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        UndoUseCase {
            storage,
            ..Default::default()
        }
    }

}

