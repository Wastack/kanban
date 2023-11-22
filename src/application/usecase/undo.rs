use crate::{IssueStorage, Presenter};
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::{UndoableHistoryElement};

#[derive(Default)]
pub(crate) struct UndoUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>,
}

impl UndoUseCase {
    pub(crate) fn execute(&mut self) -> DomainResult<()> {
        let mut board = self.storage.load();

        let history = board.history();

        let event_to_undo = history
            .peek()
            .ok_or(DomainError::new("History is empty"))?;

        match event_to_undo {
            UndoableHistoryElement::Add => {
                // TODO too much intimacy with the field
                // Board delete method puts field to deleted, which is not correct in this case

                let issues = board.issues_mut();
                issues.remove(0);
            },
            UndoableHistoryElement::Delete(info) => {
                if board.get_deleted_issues().len() < info.deletions.len() {
                    return Err(DomainError::new(&format!("The Board is in an inconsistent state: has {} deleted issues, and history suggests to restore {} deleted issues",
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
                // TODO
                return Err(DomainError::new("Not implemented"))
            },
            UndoableHistoryElement::Edit(_) => {
                // TODO
                return Err(DomainError::new("Not implemented"))
            },
            UndoableHistoryElement::Move(_) => {
                // TODO
                return Err(DomainError::new("Not implemented"))
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
    use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, UndoableHistoryElement};
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
        assert!(result.is_ok(), "{}", result.unwrap_err().description());

        then_board_for(&undo_use_case)
            .has_number_of_issues(4)
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
        assert!(result.is_ok(), "{}", result.unwrap_err().description());

        then_board_for(&undo_use_case)
            .has_number_of_issues(4)
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
        assert!(result.is_ok(), "{}", result.unwrap_err().description());

        then_board_for(&undo_use_case)
            .has_number_of_issues(4)
            .has_the_original_4_issues_in_order()
            .has_original_history();
    }

    #[test]
    fn test_undo_on_empty_board() {
        let mut undo_use_case = given_undo_usecase_with( Board::default() );
        let result =undo_use_case.execute();
        assert!(result.is_err())
    }

    #[ignore]
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
        assert!(result.is_ok(), "{}", result.unwrap_err().description());

        then_board_for(&undo_use_case)
            .has_the_original_4_issues()
            .has_additional_issue_added_with_state_open()
            .has_the_addition_in_history();

        // When undoing addition
        let result = undo_use_case.execute();
        assert!(result.is_ok(), "{}", result.unwrap_err().description());

        then_board_for(&undo_use_case)
            .has_the_original_4_issues()
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
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().description(), "The Board is in an inconsistent state: has 2 deleted issues, and history suggests to restore 3 deleted issues", "Expect proper error message");
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

        fn with_most_priority_issue_moved_to_review(mut self) -> Self {
            self.move_issue(0, State::Review).unwrap();

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

        fn has_original_history(&self) -> &Self {
            assert!(self.history().peek().is_none(), "Expected history to be empty, got: {:?}", self.history());
            self
        }
        fn has_additional_issue_added_with_state_open(&self) -> &Self {
            let Ok(issue ) = self.get_issue(0) else {panic!("Expected to have an issue")};
            assert_eq!(issue.description(), &Description::from("Additional Issue"), "Expected Additional Issue in first place");
            assert_eq!(issue.state, State::Open, "Expected issue to be in Open state");

            self
        }

        fn has_the_addition_in_history(&self) -> &Self {
            assert_eq!(self.history().peek(), Some(&UndoableHistoryElement::Add), "Expected addition to be present in history as last event");

            self
        }

    }

    fn then_board_for(undo: &UndoUseCase) -> Board {
        undo.storage.load()
    }

    fn given_undo_usecase_with(board: Board) -> UndoUseCase {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        UndoUseCase {
            storage: Box::new(storage),
            presenter: Box::new(NilPresenter::default()),
        }
    }

}

