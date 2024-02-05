#[cfg(test)]
pub mod test {
    use crate::application::{Board, Issue};
    use crate::{IssueStorage};


    #[derive(Default)]
    pub(crate) struct MemoryIssueStorage {
        pub(crate) board: Board<Issue>
    }


    impl IssueStorage for MemoryIssueStorage {
        fn load(&self) -> Board<Issue> {
            return self.board.clone();
        }

        fn save(&mut self, board: &Board<Issue>) {
            self.board = board.clone();
        }
    }
}

