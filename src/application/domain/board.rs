use std::collections::{HashMap};
use std::hash::Hash;
use nonempty_collections::{NEVec};
use crate::application::issue::{Entity, Issue};
use validated::Validated;
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::issue::State;



#[derive(Debug, PartialEq, Clone)]
pub struct Board<T: Hash + Historized> {
    entities: Vec<Entity<T>>,
    deleted_entities: Vec<Entity<T>>,
    history: Vec<T::HistoryType>,
}

/// Defines what is the type that is used to define history elements in the board.
pub trait Historized {
    type HistoryType;
}

impl<T: Hash + Historized> Default for Board<T> {
    // Because of the generic type, derive for `Default` didn't work
    fn default() -> Self {
        Self {
            entities: Default::default(),
            deleted_entities: Default::default(),
            history: Default::default(),
        }
    }
}


impl<T: Hash + Historized> Board<T> {
    pub(crate) fn new(entities: Vec<T>, deleted_entities: Vec<T>, history: Vec<T::HistoryType>) -> Self {
        Self {
            entities: entities.into_iter().map(|x| x.into()).collect(),
            deleted_entities: deleted_entities.into_iter().map(|x| x.into()).collect(),
            history,
        }
    }

    pub(crate) fn entities(&self) -> &[Entity<T>] {
        &self.entities
    }

    pub(crate) fn entity_count(&self) -> usize {
        self.entities.len()
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
        self.deleted_entities.insert(0, entity);
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

    pub fn contains(&self, index: usize) -> bool {
        self.entities.len() > index
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
        &self.deleted_entities
    }

    pub fn get_deleted_entities_mut(&mut self) -> &mut Vec<Entity<T>> {
        &mut self.deleted_entities
    }

    pub fn push_to_history(&mut self, elem: T::HistoryType) {
        self.history.push(elem)
    }

    pub fn last_history(&self) -> Option<&T::HistoryType> {
        self.history.last()
    }

    pub fn pop_history(&mut self) -> Option<T::HistoryType> {
        self.history.pop()
    }

    pub fn history(&self) -> &[T::HistoryType] {
        &self.history
    }
}

impl Board<Issue> {
    /// Changes the priority (order) of the issues, so that it becomes the most priority in
    /// its category (amongst issues with similar state).
    /// Returns the new position of the issue
    pub fn prio_top_in_category(&mut self, id: u64) -> usize {
        let state = self.get(id).state;
        let most_prio_position = self.entities
            .iter()
            .position(|i|i.state == state)
            .unwrap();

        let issue = self.remove(id);
        self.insert(most_prio_position, issue);

        most_prio_position
    }

    /// Changes the priority (order) of the issues, so that it becomes the least priority in
    /// its category (amongst issues with similar state)
    pub fn prio_bottom_in_category(&mut self, id: u64) {
        let state = self.get(id).state;
        let least_prio_position = self.entities
            .iter()
            .rposition(|i|i.state == state)
            .unwrap();

        let issue = self.remove(id);
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
    use assert2::{check, let_assert};
    use validated::Validated::{Fail, Good};
    use crate::application::issue::Description;
    use super::*;

    #[test]
    fn test_verify_indices_valid() {
        let board = given_board_with_2_tasks();
        let indices = vec![0, 1];

        let result = board.find_entities_by_indices(&indices);

        let_assert!(Good(ids) = result, "Expected validation to succeed");
        check!(ids == [4169611935799584098, 10033970510661967047]);
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
            deleted_entities: Default::default(),
            history: Default::default(),
        }
    }

    #[test]
    fn test_verify_indices_invalid() {
        let board = given_board_with_2_tasks();
        let indices = vec![0, 1, 2, 3];

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
        let indices = vec![];

        let validated = board.find_entities_by_indices(&indices);

        let_assert!(Good(ids) = validated, "Expected validated indices to be good");
        check!(ids == [] as [u64; 0]);
    }

    #[test]
    fn test_verify_indices_empty_board() {
        let board : Board<Issue>= Board::default();
        let indices = vec![0, 1, 2, 3];
        let validated = board.find_entities_by_indices(&indices);

        assert!(validated.is_fail(), "Expected validation of indices to fail");
        let Fail(errors) = validated else { panic!() };
        assert_eq!(errors.len(), 4, "Expected 4 errors for empty board");
    }

    #[test]
    fn test_prio_top_in_category_only_one_in_category() {
        let mut board = given_board_with_2_tasks(); // 0 in Open, 1 in Review

        // When
        board.prio_top_in_category(board.find_entity_id_by_index(1).unwrap());

        // Then
        check!(board.entities == given_board_with_2_tasks().entities, "Expect board not to change");
    }

    #[test]
    fn test_prio_top_in_category() {
        let mut board = Board::new(
            vec![
                Issue { description: Description::from("First open task"), state: State::Open, time_created: 0 },
                Issue { description: Description::from("First done task"), state: State::Done, time_created: 0 },
                Issue { description: Description::from("First review task"), state: State::Review, time_created: 0 },
                Issue { description: Description::from("Second open task"), state: State::Open, time_created: 0 },
                Issue { description: Description::from("Third open task"), state: State::Open, time_created: 0 },
            ],
            vec![],
            vec![]);

        // When
        let id_of_third_open_task = board.find_entity_id_by_index(4).unwrap();
        board.prio_top_in_category(id_of_third_open_task); // Third open task

        // Then
        [
            ("Third open task", State::Open),
            ("First open task", State::Open),
            ("First done task", State::Done),
            ("First review task", State::Review),
            ("Second open task", State::Open),
        ].into_iter().enumerate().for_each(|(index, (expected_description, expected_state))| {
            let entity = &board.entities()[index];
            check!(entity.description == Description::from(expected_description), "Expected specific description for Issue at index '{}'", index);
            check!(entity.state == expected_state, "Expected specific state for Issue at index '{}'", index);
        });
    }
}