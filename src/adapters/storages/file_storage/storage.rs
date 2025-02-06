use std::fs;
use std::io::Write;
use std::path::PathBuf;
use home::home_dir;
use crate::adapters::storages::file_storage::serde_resources::StoredBoard;
use crate::adapters::storages::IssueStorage;
use crate::application::Issue;
use crate::application::domain::historized_board::HistorizedBoard;

pub struct FileStorage {
    pub source: PathBuf,
}

impl Default for FileStorage {
    fn default() -> Self {
        FileStorage {
            source: home_dir().expect("Failed to get home directory")
                .join(".kanban").into(),
        }
    }
}

impl IssueStorage for FileStorage {
    fn load(&self) -> HistorizedBoard<Issue> {
        let file_contents = fs::read_to_string(&self.source)
            .unwrap_or_default();

        if file_contents.is_empty() {
            return HistorizedBoard::default();
        }

        let stored_board = serde_yaml::from_str::<StoredBoard>(&file_contents)
            .expect("unexpected file format");

        stored_board.into()
    }

    fn save(&self, board: &HistorizedBoard<Issue>) {
        let content = Self::board_to_yaml(board);

        let mut file = fs::File::create(&self.source).expect("cannot open file to write board");
        file.write_all(content.as_bytes()).expect("cannot write to file");
    }
}

impl FileStorage {
    fn board_to_yaml(board: &HistorizedBoard<Issue>) -> String {
        let storable_board = StoredBoard::from(board);
        let content = serde_yaml::to_string(&storable_board)
            .expect("Internal error: cannot serialize board");
        content
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::ops::Deref;
    use assert2::check;
    use time::macros::date;
    use crate::application::{Issue, State};
    use crate::adapters::storages::file_storage::FileStorage;
    use crate::adapters::storages::IssueStorage;
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, EditHistoryElement, MoveHistoryElement, MoveHistoryElements, PrioHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;

    #[test]
    fn test_file_storage_load_non_existent_file_failed_no_permission() {
        let storage = FileStorage {
            source: current_dir().unwrap().join("resources/test/example_board.yaml")
        };

        let board = storage.load();
        // Then
        check!(board.entity_count() == 2, "Expected board to have two issues");
        check!(board.get_deleted_entities().len() == 2, "Expected board to have 2 deleted issues");
        check!(board.history.stack.len() == 7, "Expected board to have a specific number of history elements");
        [
            Issue {
                description: Description::from("Get a coffee"),
                state: State::Open,
                time_created: date!(2024-01-31),
                due_date: None,
            },
            Issue {
                description: Description::from("Take a break"),
                state: State::Done,
                time_created: date!(2023-12-11),
                due_date: None,
            },
        ].into_iter().zip(board.entities().iter()).for_each(|(expected_issue, actual_issue)| {
            assert_eq!(actual_issue.deref(), &expected_issue, "Expected specific loaded issues")
        });

        let expected_history = vec![
            UndoableHistoryElement::Add,
            UndoableHistoryElement::Edit(EditHistoryElement{
                original_description: String::from("Don't get a coffee"),
                index: 0,
            }),
            UndoableHistoryElement::Delete(DeleteHistoryElements{
                deletions: vec![
                    DeleteHistoryElement{ original_position_in_issues: 2 },
                    DeleteHistoryElement{ original_position_in_issues: 3 },
                ],
            }),
            UndoableHistoryElement::Add,
            UndoableHistoryElement::Add,
            UndoableHistoryElement::Prio(PrioHistoryElement{
                original_index: 1,
                new_index: 0,
            }),
            UndoableHistoryElement::Move(MoveHistoryElements{
                moves: vec![MoveHistoryElement{
                    original_index: 1,
                    original_state: State::Open,
                    new_index: 1,
                }],
            }),
        ];

        check!(board.history.stack == expected_history.as_slice(), "Expected specific history");
    }

    #[test]
    fn test_file_storage_load_non_existent_file_successful() {
        // Given
        let storage = FileStorage {
            source: "/tmp/non_existent".into()
        };

        // When
        let board = storage.load();

        // Then
        check_boards_are_equal(&board, &HistorizedBoard::default())
    }

    #[test]
    fn test_typical_board_to_storage_yaml() {
        let board = HistorizedBoard::default().with_4_typical_issues();
        let formatted_output  = FileStorage::board_to_yaml(&board);

        assert_eq!(formatted_output,r#"---
issues:
  - description: Task inserted fourth
    state: open
    timeCreated: 2025-02-10
    dueDate: ~
  - description: Task inserted third
    state: done
    timeCreated: 2025-02-03
    dueDate: ~
  - description: Task inserted second
    state: review
    timeCreated: 2025-02-12
    dueDate: 2025-02-16
  - description: Task inserted first
    state: open
    timeCreated: 2025-02-13
    dueDate: ~
deletedIssues: []
history: []
"#);
    }
}
