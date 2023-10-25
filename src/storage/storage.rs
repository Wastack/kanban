use std::fs;
use std::io::Write;
use std::path::{PathBuf};
use home::home_dir;
use crate::{Board};

pub trait Storage {
    fn load(&self) -> Board;
    fn save(&self, board: &Board);
}

pub struct FileStorage {
    pub source: PathBuf
}

pub fn home_file_storage() -> FileStorage {
    FileStorage{
        source: home_dir().expect("Failed to get home directory")
            .join(".kanban").into(),
    }
}


impl Storage for FileStorage {
    fn load(&self) -> Board {
        let file_contents = fs::read_to_string(&self.source)
            .unwrap_or(String::from(""));

        if file_contents == "" {
            return Board{
                issues: vec![]
            }
        }

        serde_yaml::from_str(&file_contents).expect("unexpected file format")
    }

    fn save(&self, board: &Board) {
        let content = serde_yaml::to_string(board)
            .expect("Internal error: cannot serialize board");

        let mut file = fs::File::create(&self.source).expect("cannot open file to write board");
        file.write_all(content.as_bytes()).expect("cannot write to file");
    }
}