use std::collections::HashMap;
use crate::model::issue::Issue;
use serde::{Serialize, Deserialize};
use crate::model::issue::State;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub issues: Vec<Issue>,
}

impl Board {
    /// Changes the priority (order) of the issues, so that it becomes the most priority in
    /// its category (amongst issues with similar state)
    pub fn prio_top_in_category(&mut self, index: usize) {
        let state = self.issues[index].state;
        let most_prio_position = self.issues
            .iter()
            .position(|i|i.state == state)
            .unwrap();

        let issue = self.issues.remove(index);
        self.issues.insert(most_prio_position, issue);
    }

    /// Changes the priority (order) of the issues, so that it becomes the least priority in
    /// its category (amongst issues with similar state)
    pub fn prio_bottom_in_category(&mut self, index: usize) {
        let state = self.issues[index].state;
        let least_prio_position = self.issues
            .iter()
            .rposition(|i|i.state == state)
            .unwrap();

        let issue = self.issues.remove(index);
        // the previous remove modified the positions, thus no +1 needed
        self.issues.insert(least_prio_position, issue);
    }

    /// Changes the priority (order) of the issues, so that it becomes one more priority in
    /// its category (amongst issues with similar state)
    pub fn prio_up_in_category(&mut self, index: usize) {
        let state = self.issues[index].state;
        let issues = self.issues_with_state();
        let category = issues.get(&state).unwrap();
        let position_in_category = category
            .iter()
            .position(|i|i.order == index)
            .unwrap();

        if position_in_category <= 0 {
            // there is nothing to do, because the issue is already top priority
            return
        }

        let position_of_issue_above = category
            .get(position_in_category-1)
            .unwrap()
            .order;

        let issue = self.issues.remove(index);
        self.issues.insert(position_of_issue_above, issue);
    }

    /// Changes the priority (order) of the issues, so that it one less priority in
    /// its category (amongst issues with similar state)
    pub fn prio_down_in_category(&mut self, index: usize) {
        let state = self.issues[index].state;
        let issues = self.issues_with_state();
        let category = issues.get(&state).unwrap();
        let position_in_category = category
            .iter()
            .position(|i|i.order == index)
            .unwrap();

        if position_in_category >= category.len() - 1 {
            // there is nothing to do, because the issue is already least priority
            return
        }

        let position_of_issue_below = category
            .get(position_in_category+1)
            .unwrap()
            .order;

        let issue = self.issues.remove(index);

        // the previous remove modified the positions, thus no +1 needed
        self.issues.insert(position_of_issue_below, issue);
    }
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

/// And Issue referenced, and its position (a.k.a. order or priority) amongst all the issues.
pub struct IssueRef<'a> {
    pub order: usize,
    pub issue: &'a Issue,
}

pub trait BoardStateView {
    fn issues_with_state(&self) -> HashMap<State, Vec<IssueRef>>;
}