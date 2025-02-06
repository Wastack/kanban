use crate::application::Issue;
use crate::application::domain::historized_board::HistorizedBoard;

pub trait IssueStorage {
    fn load(&self) -> HistorizedBoard<Issue>;
    fn save(&self, board: &HistorizedBoard<Issue>);
}