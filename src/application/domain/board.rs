use std::collections::{HashMap};
use std::ops::{Deref, DerefMut};
use nonempty_collections::{NEVec};
use crate::application::issue::{Entity, Issue, Stateful};
use serde::{Deserialize, Serialize};
use validated::Validated;
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::History;
use crate::application::issue::State;



#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    #[serde(default)]
    issues: Vec<Entity<Issue>>,

    #[serde(default)]
    deleted_issues: Vec<Entity<Issue>>,

    #[serde(default)]
    history: History,
}

impl Board {
    pub(crate) fn entities(&self) -> &[Entity<Issue>] {
        &self.issues
    }

    /// Gets entity with id. Panics if used with a non-existing id
    pub(crate) fn get_entity(&self, id: u64) -> &Entity<Issue> {
        self.issues.iter()
            .find(|&entity| entity.id == id)
            .unwrap()
    }

    /// Removes entity with id. Panics if used with a non-existing id
    pub(crate) fn remove_entity(&mut self, id: u64) -> Entity<Issue> {
        let index = self.issues.iter().position(|entity| entity.id == id).unwrap();
        self.issues.remove(index)
    }

    pub(crate) fn remove(&mut self, index: usize) -> Entity<Issue> {
        self.issues.remove(index)
    }

    pub fn find_entity_id_by_issue_order(&self, order: usize) -> DomainResult<u64> {
        self.issues.get(order)
            .and_then(|e| Some(e.id))
            .ok_or(DomainError::IndexOutOfRange(order))
    }

    pub fn get_issue(&self, index: usize) -> DomainResult<&Entity<Issue>> {
        self.issues.get(index).ok_or(DomainError::IndexOutOfRange(index))
    }

    /// Returns the number of (not deleted) issues in Board
    pub fn issues_count(&self) -> usize {
        self.issues.len()
    }

    pub fn get_issue_mut(&mut self, index: usize) -> DomainResult<&mut Issue> {
        self.issues.get_mut(index)
            .and_then(|x|Some(x.deref_mut()))
                .ok_or(DomainError::IndexOutOfRange(index))
    }

    pub fn contains(&self, index: usize) -> bool {
        self.issues.len() > index
    }

    pub fn validate_indices(&self, indices: &[usize]) -> Validated<(), DomainError> {
        let mut errors = indices
            .iter()
            .filter(|&&i| !self.contains(i))
            .map(|&i|DomainError::IndexOutOfRange(i));

        match errors.next() {
            None => Validated::Good(()),
            Some(head) => Validated::Fail(NEVec::from((head, errors.collect())))
        }
    }

    /// Delete issues with the given indices, in the same order.
    ///
    /// Deleted issues are still accessible with `get_deleted_issues`.
    /// Order of the deleted issues is relevant.
    ///
    /// If indices are [2, 0, 1], then the most recently deleted issue is with index `1`, because
    /// it first deletes `2`, then `0` and then `1`.
    pub fn delete_issues_with(&mut self, indices: &[usize]) {
        // Sort the indices in descending order,
        // so that each removal does not affect the next index.
        let mut sorted_descending_indices = indices.to_owned();
        sorted_descending_indices.sort_unstable_by(|a, b| b.cmp(a));

        let mut removed_issues: HashMap<usize, Entity<Issue>> = sorted_descending_indices
            .into_iter()
            .map(|index|(index, self.issues.remove(index)))
            .collect();

        for i in indices {
            self.deleted_issues.insert(0, removed_issues.remove(i).unwrap());
        }
    }

    /// Adds a new issue to the board to first priority
    pub fn append_issue(&mut self, issue: Issue) {
        self.issues.insert( 0, issue.into() );
    }

    pub fn insert(&mut self, index: usize, entity: Entity<Issue>) {
        self.issues.insert(index, entity)
    }

    /// Move an issue from one state to another
    pub fn move_issue(&mut self, index: usize, new_state: State) -> DomainResult<()> {
        let state = self.get_issue_mut(index)?.state_mut();

        if *state == new_state {
            return Ok(());
        }

        *state = new_state;

        Ok(())
    }

    /// Returns a list of the deleted issues. The first element of the list is the one most recently deleted.
    pub fn get_deleted_issues(&self) -> &[Entity<Issue>] {
        &self.deleted_issues
    }

    pub fn get_deleted_issues_mut(&mut self) -> &mut Vec<Entity<Issue>> {
        &mut self.deleted_issues
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }
}

impl Board {
    /// Changes the priority (order) of the issues, so that it becomes the most priority in
    /// its category (amongst issues with similar state).
    /// Returns the new position of the issue
    pub fn prio_top_in_category(&mut self, index: usize) -> usize {
        let state = self.issues[index].state;
        let most_prio_position = self.issues
            .iter()
            .position(|i|i.state == state)
            .unwrap();

        let issue = self.issues.remove(index);
        self.issues.insert(most_prio_position, issue);

        most_prio_position
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
    /// returned Vectors are ordered by their priority.
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


#[cfg(test)]
mod tests {
    use validated::Validated::Fail;
    use crate::application::issue::Description;
    use super::*;

    #[test]
    fn test_verify_indices_valid() {
        let board = given_board_with_2_tasks();
        let indices = given_indices_within_bounds();

        let result = board.validate_indices(&indices);

        assert!(result.is_good(), "Expected validation to be good");
    }

    fn given_board_with_2_tasks() -> Board {
        Board {
            issues: vec![
                Issue {
                    description: Description::from("First task"),
                    state: State::Open,
                    time_created: 1698397489,

                }.into(),
                Issue {
                    description: Description::from("Second task"),
                    state: State::Review,
                    time_created: 1698397490,

                }.into(),
            ],
            deleted_issues: Vec::default(),
            history: History::default(),
        }
    }

    #[test]
    fn test_verify_indices_invalid() {
        let board = given_board_with_2_tasks();
        let indices = given_some_indices_are_out_of_range();

        let validated = board.validate_indices(&indices);


        assert!(validated.is_fail(), "Expected validation to fail");
        let Fail(errors) = validated else { panic!("expected error when validating indices of board") };
        assert_eq!(errors.len(), 2, "Expected 2 errors");
        assert!(matches!(errors[0], DomainError::IndexOutOfRange(2)));
        assert!(matches!(errors[1], DomainError::IndexOutOfRange(3)));
    }

    #[test]
    fn test_verify_indices_empty_indices() {
        let board = given_board_with_2_tasks();
        let indices = given_no_indices();

        let validated = board.validate_indices(&indices);

        assert!(validated.is_good(), "Expected validated indices to be good");
    }

    #[test]
    fn test_verify_indices_empty_board() {
        let board = given_empty_board();
        let indices = given_some_indices_are_out_of_range();
        let validated = board.validate_indices(&indices);

        assert!(validated.is_fail(), "Expected validation of indices to fail");
        let Fail(errors) = validated else { panic!() };
        assert_eq!(errors.len(), 4, "Expected 4 errors for empty board");
    }

    fn given_empty_board() -> Board {
        Board::default()
    }

    fn given_indices_within_bounds() -> Vec<usize> {
        vec![0, 1]
    }

    fn given_some_indices_are_out_of_range() -> Vec<usize> {
        vec![0, 1, 2, 3]
    }

    fn given_no_indices() -> Vec<usize> {
        vec![]
    }
}