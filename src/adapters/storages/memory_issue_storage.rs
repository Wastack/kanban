#[cfg(test)]
pub mod test {
    use crate::application::{HistorizedBoard, Issue};
    use crate::{IssueStorage};


    #[derive(Default)]
    pub(crate) struct MemoryIssueStorage {
        pub(crate) board: HistorizedBoard<Issue>
    }


    impl IssueStorage for MemoryIssueStorage {
        fn load(&self) -> HistorizedBoard<Issue> {
            return self.board.clone();
        }

        fn save(&mut self, board: &HistorizedBoard<Issue>) {
            self.board = board.clone();
        }
    }
}

