use crate::application::Board;

pub trait IssueStorage {
    fn load(&self) -> Board;
    fn save(&self, board: &Board);
}