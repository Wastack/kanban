use std::collections::{HashMap};
use std::hash::Hash;
use std::ops::{DerefMut};
use nonempty_collections::{NEVec};
use crate::application::issue::{Entity, Issue, Stateful};
use validated::Validated;
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::domain::history::History;
use crate::application::issue::State;



#[derive(Debug, PartialEq, Clone)]
pub struct Board<T: Hash> {
    pub(crate) entities: Vec<Entity<T>>,
    pub(crate) deleted_issues: Vec<Entity<T>>,
    // TODO: as long as the `History` is not generic, Board is not independent of `Issue`
    pub(crate) history: History,
}

impl<T: Hash> Default for Board<T> {
    // Because of the generic type, derive for `Default` didn't work
    fn default() -> Self {
        Self {
            entities: Default::default(),
            deleted_issues: Default::default(),
            history: Default::default(),
        }
    }
}


impl<T: Hash> Board<T> {
    pub(crate) fn entities(&self) -> &[Entity<T>] {
        &self.entities
    }

    /// Gets entity with id. Panics if used with a non-existing id
    pub(crate) fn get(&self, id: u64) -> &Entity<T> {
        self.entities.iter()
            .find(|&entity| entity.id == id)
            .unwrap()
    }

    pub(crate) fn get_mut(&mut self, id: u64) -> &mut Entity<T> {
        self.entities.iter_mut()
            .find(|entity| entity.id == id)
            .unwrap()
    }

    /// Removes entity with id. Panics if used with a non-existing id.
    pub(crate) fn remove(&mut self, id: u64) -> Entity<T> {
        let index = self.entities.iter().position(|entity| entity.id == id).unwrap();
        self.entities.remove(index)
    }

    pub(crate) fn mark_as_deleted(&mut self, id: u64) {
        let index = self.entities.iter().position(|entity| entity.id == id).unwrap();
        let entity = self.entities.remove(index);
        self.deleted_issues.insert(0, entity);
    }

    pub(crate) fn remove_by_index(&mut self, index: usize) -> Entity<T> {
        self.entities.remove(index)
    }

    pub fn find_entity_id_by_index(&self, index: usize) -> DomainResult<u64> {
        self.entities.get(index)
            .and_then(|e| Some(e.id))
            .ok_or(DomainError::IndexOutOfRange(index))
    }

    pub fn find_entities_by_indices(&self, indices: &[usize]) -> Validated<Vec<u64>, DomainError> {
        let mut errors = indices
            .iter()
            .filter(|&&i| !self.contains(i))
            .map(|&i|DomainError::IndexOutOfRange(i));

        if let Some(head) = errors.next() {
            return Validated::Fail(NEVec::from((head, errors.collect())));
        }

        Validated::Good(indices.iter().map(|&order| self.entities[order].id).collect())
    }

    pub fn get_by_index(&self, index: usize) -> DomainResult<&Entity<T>> {
        /// TODO: this method should go away
        self.entities.get(index).ok_or(DomainError::IndexOutOfRange(index))
    }

    /// Returns the number of (not deleted) issues in Board
    pub fn issues_count(&self) -> usize {
        self.entities.len()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.entities.len() > index
    }

    /// Delete issues with the given indices, in the same order.
    ///
    /// Deleted issues are still accessible with `get_deleted_issues`.
    /// Order of the deleted issues is relevant.
    ///
    /// If indices are [2, 0, 1], then the most recently deleted issue is with index `1`, because
    /// it first deletes `2`, then `0` and then `1`.
    /// TODO: this method should go away
    pub fn delete_entities_by_indices(&mut self, indices: &[usize]) {
        // Sort the indices in descending order,
        // so that each removal does not affect the next index.
        let mut sorted_descending_indices = indices.to_owned();
        sorted_descending_indices.sort_unstable_by(|a, b| b.cmp(a));

        let mut removed_entities: HashMap<usize, Entity<T>> = sorted_descending_indices
            .into_iter()
            .map(|index|(index, self.entities.remove(index)))
            .collect();

        for i in indices {
            self.deleted_issues.insert(0, removed_entities.remove(i).unwrap());
        }
    }

    /// Adds a new issue to the board to first priority
    pub fn append_entity(&mut self, issue: T) {
        self.entities.insert(0, issue.into() );
    }

    pub fn insert(&mut self, index: usize, entity: Entity<T>) {
        self.entities.insert(index, entity)
    }

    /// Returns a list of the deleted issues. The first element of the list is the one most recently deleted.
    pub fn get_deleted_entities(&self) -> &[Entity<T>] {
        &self.deleted_issues
    }

    pub fn get_deleted_entities_mut(&mut self) -> &mut Vec<Entity<T>> {
        &mut self.deleted_issues
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }
}

impl Board<Issue> {
    /// Changes the priority (order) of the issues, so that it becomes the most priority in
    /// its category (amongst issues with similar state).
    /// Returns the new position of the issue
    pub fn prio_top_in_category(&mut self, index: usize) -> usize {
        let state = self.entities[index].state;
        let most_prio_position = self.entities
            .iter()
            .position(|i|i.state == state)
            .unwrap();

        let issue = self.entities.remove(index);
        self.entities.insert(most_prio_position, issue);

        most_prio_position
    }

    /// Changes the priority (order) of the issues, so that it becomes the least priority in
    /// its category (amongst issues with similar state)
    pub fn prio_bottom_in_category(&mut self, index: usize) {
        let state = self.entities[index].state;
        let least_prio_position = self.entities
            .iter()
            .rposition(|i|i.state == state)
            .unwrap();

        let issue = self.entities.remove(index);
        // the previous remove modified the positions, thus no +1 needed
        self.entities.insert(least_prio_position, issue);
    }

    /// Changes the priority (order) of the issues, so that it becomes one more priority in
    /// its category (amongst issues with similar state)
    pub fn prio_up_in_category(&mut self, index: usize) {
        let state = self.entities[index].state;
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

        let issue = self.entities.remove(index);
        self.entities.insert(position_of_issue_above, issue);
    }

    /// Changes the priority (order) of the issues, so that it one less priority in
    /// its category (amongst issues with similar state)
    pub fn prio_down_in_category(&mut self, index: usize) {
        let state = self.entities[index].state;
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

        let issue = self.entities.remove(index);

        // the previous remove modified the positions, thus no +1 needed
        self.entities.insert(position_of_issue_below, issue);
    }
}


impl BoardStateView for Board<Issue> {
    /// Returns the issues categorized by state, alongside their global order (priority). The
    /// returned Vectors are ordered by their priority.
    fn issues_with_state(&self) -> HashMap<State, Vec<IssueRef>> {
        self.entities.iter()
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

        let result = board.find_entities_by_indices(&indices);

        assert!(result.is_good(), "Expected validation to be good");
        // TODO: assert ids
    }

    fn given_board_with_2_tasks() -> Board<Issue> {
        Board {
            entities: vec![
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

        let validated = board.find_entities_by_indices(&indices);


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

        let validated = board.find_entities_by_indices(&indices);

        assert!(validated.is_good(), "Expected validated indices to be good");
        // TODO: validate ids
    }

    #[test]
    fn test_verify_indices_empty_board() {
        let board = given_empty_board();
        let indices = given_some_indices_are_out_of_range();
        let validated = board.find_entities_by_indices(&indices);

        assert!(validated.is_fail(), "Expected validation of indices to fail");
        let Fail(errors) = validated else { panic!() };
        assert_eq!(errors.len(), 4, "Expected 4 errors for empty board");
    }

    fn given_empty_board() -> Board<Issue> {
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