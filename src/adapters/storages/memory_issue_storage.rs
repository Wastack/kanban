#[cfg(test)]

use crate::application::Board;
use crate::{IssueStorage};


#[derive(Default)]
pub(crate) struct MemoryIssueStorage {
    board: Board
}


impl IssueStorage for MemoryIssueStorage {
    fn load(&self) -> Board {
        return self.board.clone();
    }

    fn save(&mut self, board: &Board) {
        self.board = board.clone();
    }
}
