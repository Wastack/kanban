#[cfg(test)]
pub mod test {
    use std::cell::{RefCell};
    use crate::adapters::storages::IssueStorage;
    use crate::application::Issue;
    use crate::application::domain::historized_board::HistorizedBoard;


    #[derive(Default)]
    pub(crate) struct MemoryIssueStorage {
        pub(crate) board: RefCell<HistorizedBoard<Issue>>
    }


    impl IssueStorage for MemoryIssueStorage {
        fn load(&self) -> HistorizedBoard<Issue> {
            self.board.borrow().clone()
        }

        fn save(&self, board: &HistorizedBoard<Issue>) {
            self.board.swap(&RefCell::new(board.clone()));
        }
    }
}

