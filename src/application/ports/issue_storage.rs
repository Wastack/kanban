use crate::application::{HistorizedBoard, Issue};

pub trait IssueStorage {
    fn load(&self) -> HistorizedBoard<Issue>;
    fn save(&mut self, board: &HistorizedBoard<Issue>);
}