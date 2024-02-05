use std::fs;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use home::home_dir;
use serde::{Deserialize, Serialize};
use crate::adapters::storages::file_storage::serde_resources::StoredBoard;
use crate::application::Board;
use crate::application::issue::Described;
use crate::IssueStorage;

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
    fn load(&self) -> Board {
        let file_contents = fs::read_to_string(&self.source)
            .unwrap_or_default();

        if file_contents.is_empty() {
            return Board::default();
        }

        let stored_board = serde_yaml::from_str::<StoredBoard>(&file_contents)
            .expect("unexpected file format");

        stored_board.into()
    }

    fn save(&mut self, board: &Board) {
        let content = Self::board_to_string(board);

        let mut file = fs::File::create(&self.source).expect("cannot open file to write board");
        file.write_all(content.as_bytes()).expect("cannot write to file");
    }
}

impl FileStorage {
    fn board_to_string(board: &Board) -> String {
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
    use crate::application::{Board, Issue, State};
    use crate::IssueStorage;
    use crate::adapters::storages::file_storage::FileStorage;
    use crate::application::domain::history::{History, UndoableHistoryElement};
    use crate::application::issue::Description;

    #[test]
    fn test_file_storage_load_non_existent_file_failed_no_permission() {
        let storage = FileStorage {
            source: current_dir().unwrap().join("resources/test/example_board.yaml")
        };

        let board = storage.load();
        // Then
        assert_eq!(board.issues_count(), 2, "Expected board to have two issues");
        assert_eq!(board.get_deleted_issues().len(), 2, "Expected board to have 2 deleted issues");
        assert_eq!(board.history().len(), 2, "Expected board to have a history of 2 items");
        [
            Issue {
                description: Description::from("Get a coffee"),
                state: State::Open,
                time_created: 1706727855,
            },
            Issue {
                description: Description::from("Take a break"),
                state: State::Done,
                time_created: 1702298969,
            },
        ].into_iter().zip(board.entities().iter()).for_each(|(expected_issue, actual_issue)| {
            assert_eq!(actual_issue.deref(), &expected_issue, "Expected specific loaded issues")
        });
        // TODO: add more sophisticated history elements to test
        let expected_history = History::default()
            .with_element(UndoableHistoryElement::Add)
            .with_element(UndoableHistoryElement::Add);

        assert_eq!(board.history(), &expected_history, "Expected specific history");
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
        assert_eq!(board, Board::default());
    }


    // TODO: validate dynamic aspects of Board after loading, like:
    // - Whether undo entries are consistent with the rest of the board:
    //   + undo add, but there is no issue in Board
    //   + undo delete, but there is no history
    //   + undo move, but there is no ticket in where the ticket was supposed to be moved
    // ...
}
