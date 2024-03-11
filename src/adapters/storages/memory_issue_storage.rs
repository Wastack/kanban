#[cfg(test)]
pub mod test {
    use crate::adapters::storages::IssueStorage;
    use crate::application::Issue;
    use crate::application::domain::historized_board::HistorizedBoard;


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

