use crate::application::{Board, Issue};

pub trait IssueStorage {
    fn load(&self) -> Board<Issue>;
    fn save(&mut self, board: &Board<Issue>);
}