use std::fs;
use std::io::Write;
use std::path::PathBuf;
use home::home_dir;
use crate::application::Board;
use crate::IssueStorage;

pub struct FileStorage {
    pub source: PathBuf
}

impl Default for FileStorage {
    fn default() -> Self {
        FileStorage{
            source: home_dir().expect("Failed to get home directory")
                .join(".kanban").into(),
        }
    }
}

impl IssueStorage for FileStorage {
    fn load(&self) -> Board {
        let file_contents= fs::read_to_string(&self.source)
            .unwrap_or_default();

        if file_contents.is_empty() {
            return Board::default();
        }

        serde_yaml::from_str(&file_contents).expect("unexpected file format")
    }

    fn save(&mut self, board: &Board) {
        let content = serde_yaml::to_string(board)
            .expect("Internal error: cannot serialize board");

        let mut file = fs::File::create(&self.source).expect("cannot open file to write board");
        file.write_all(content.as_bytes()).expect("cannot write to file");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::application::Board;
    use crate::{FileStorage, IssueStorage};
    
    struct FileCleanUp {
        path: PathBuf
    }
    
    impl Drop for FileCleanUp {
        fn drop(&mut self) {
            todo!()
        }
    }

    #[test]
    fn test_file_storage_load_non_existent_file_failed_no_permission() {
        // TODO
    }

    #[test]
    fn test_file_storage_load_non_existent_file_successful() {
        let storage = given_file_storage_with_non_existent_file();

        // When
        let board = storage.load();

        board.assert_default();
    }

    fn given_file_storage_with_non_existent_file() -> FileStorage {
        FileStorage {
            source: "/tmp/non_existent".into()
        }
    }

    impl Board {
        fn assert_default(&self) -> &Self {
            assert_eq!(self, &Board::default());

            self
        }
    }
}
