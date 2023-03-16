use std::collections::HashMap;
use crate::model::issue::Issue;
use serde::{Serialize, Deserialize};
use crate::model::issue::State;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub issues: Vec<Issue>,
}


impl BoardStateView for Board {
    /// Returns the issues categorized by state, alongside their global order (priority). The
    /// returned Vectors are ordered by their order.
    fn issues_with_state(&self) -> HashMap<State, Vec<IssueRef>> {
        self.issues.iter()
            .enumerate()
            .map(|(order, issue) | (issue.state, IssueRef{ order, issue }))
            .fold(HashMap::new(), |mut acc, (state, issue_ref) | {
                acc.entry(state).or_insert_with(Vec::new).push(issue_ref);
                acc
            })
    }
}

pub struct IssueRef<'a> {
    pub order: usize,
    pub issue: &'a Issue,
}

pub trait BoardStateView {
    fn issues_with_state(&self) -> HashMap<State, Vec<IssueRef>>;
}