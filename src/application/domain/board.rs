use std::collections::{HashMap};
use std::hash::Hash;
use std::iter;
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

    pub fn position(&self, id: u64) -> usize {
        self.entities.iter().position(|e| e.id == id).unwrap()
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
    pub fn prio_up_in_category(&mut self, id: u64) {
        let state = self.get(id).state;

        let entity_pos_reversed = self.entities
            .iter()
            .rev()
            .position(|i|i.id == id).unwrap();

        let move_up_this_much = self.entities.iter()
            .rev()
            .skip(entity_pos_reversed+1)
            .position(|i|i.state == state);

        let move_up_this_much = if let Some(p) = move_up_this_much { p + 1 } else {
            // there is nothing to do, because the issue is already top priority
            // todo: should this be an error?
            return
        };

        let new_position = self.position(id) - move_up_this_much;
        let entity = self.remove(id);
        self.insert(new_position, entity);
    }

    /// Changes the priority (order) of the issues, so that it one less priority in
    /// its category (amongst issues with similar state)
    pub fn prio_down_in_category(&mut self, id: u64) {
        let current_position =  self.position(id);

        let steps_down = self.entities.iter()
            .skip(current_position+1)
            .position(|x| x.state == self.get(id).state)
            .map(|steps| steps + 1 );

        let steps_down = if let Some(p) = steps_down { p } else {
            return
        };

        let issue = self.remove(id);

        // by removing an issue, all subsequent indices shift to the left. Thus, -1.
        self.entities.insert(current_position + steps_down, issue);
    }
}


// todo: this is only needed for presenters
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
    use std::borrow::Borrow;
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
        board.prio_top_in_category(board.find_entity_id_by_index(1).unwrap());
        check!(board.entities == given_board_with_2_tasks().entities, "Expect board not to change");
    }

    #[test]
    fn test_prio_top_in_category() {
        let mut board = board_for_testing_priorities();

        // When
        let id_of_third_open_task = board.find_entity_id_by_index(4).unwrap();
        board.prio_top_in_category(id_of_third_open_task); // Third open task

        // Then
        let expected = [
            ("Third open task", State::Open),
            ("First open task", State::Open),
            ("First done task", State::Done),
            ("First review task", State::Review),
            ("Second open task", State::Open),
        ];

        check_priorities(&expected, &board);
    }

    #[test]
    fn test_prio_bottom_in_category_solo_in_state() {
        let mut board = given_board_with_2_tasks(); // 0 in Open, 1 in Review
        board.prio_bottom_in_category(board.find_entity_id_by_index(0).unwrap());
        check!(board.entities == given_board_with_2_tasks().entities, "Expect board not to change");
    }

    #[test]
    fn test_prio_up_in_category_solo_in_state() {
        let mut board = given_board_with_2_tasks(); // 0 in Open, 1 in Review
        board.prio_up_in_category(board.find_entity_id_by_index(0).unwrap());
        check!(board.entities == given_board_with_2_tasks().entities, "Expect board not to change");
    }

    #[test]
    fn test_prio_down_in_category_solo_in_state() {
        let mut board = given_board_with_2_tasks(); // 0 in Open, 1 in Review
        board.prio_down_in_category(board.find_entity_id_by_index(0).unwrap());
        check!(board.entities == given_board_with_2_tasks().entities, "Expect board not to change");
    }

    #[test]
    fn test_prio_bottom_in_category() {
        let mut board = board_for_testing_priorities();

        // When
        let id = board.find_entity_id_by_index(0).unwrap();
        board.prio_bottom_in_category(id); // Third open task

        // Then
        let expected = [
            ("First done task", State::Done), // ^ from here
            ("First review task", State::Review),
            ("Second open task", State::Open),
            ("Third open task", State::Open),
            ("First open task", State::Open), // moved here
        ];

        check_priorities(&expected, &board);
    }

    #[test]
    fn test_prio_up_in_category() {
        let mut board = board_for_testing_priorities();

        // When
        let id = board.find_entity_id_by_index(4).unwrap();
        board.prio_up_in_category(id); // Third open task

        // Then
        let expected = [
            ("First open task", State::Open),
            ("First done task", State::Done),
            ("First review task", State::Review),
            ("Third open task", State::Open),
            ("Second open task", State::Open),
        ];

        check_priorities(&expected, &board);
    }

    #[test]
    fn test_prio_down_in_category() {
        let mut board = board_for_testing_priorities();

        // When
        let id_of_third_open_task = board.find_entity_id_by_index(3).unwrap();
        board.prio_down_in_category(id_of_third_open_task); // Third open task

        // Then
        let expected = [
            ("First open task", State::Open),
            ("First done task", State::Done),
            ("First review task", State::Review),
            ("Third open task", State::Open), // ^ from here
            ("Second open task", State::Open), // <-- moved here
        ];

        check_priorities(&expected, &board);
    }

    #[test]
    fn test_prio_up_multiple_hop () {
        let mut board = board_for_testing_priorities();

        // When
        let id = board.find_entity_id_by_index(0).unwrap();
        board.prio_down_in_category(id); // Third open task

        // Then
        let expected = [
            ("First done task", State::Done), // ^ from here
            ("First review task", State::Review),
            ("Second open task", State::Open),
            ("First open task", State::Open), // moved here
            ("Third open task", State::Open),
        ];

        check_priorities(&expected, &board);

    }

    fn check_priorities(expected: &[(&str, State)], actual: &Board<Issue>) {
        expected.into_iter().enumerate().for_each(|(index, &(expected_description, expected_state))| {
            let entity = &actual.entities()[index];
            check!(entity.description == Description::from(expected_description), "Expected specific description for Issue at index '{}.\nBoard was: '{:?}'", index, actual);
            check!(entity.state == expected_state, "Expected specific state for Issue at index '{}'.\nBoard was: '{:?}'", index, actual);
        })
    }

    fn board_for_testing_priorities() -> Board<Issue> {
        Board::new(
            vec![
                Issue { description: Description::from("First open task"), state: State::Open, time_created: 0 },
                Issue { description: Description::from("First done task"), state: State::Done, time_created: 0 },
                Issue { description: Description::from("First review task"), state: State::Review, time_created: 0 },
                Issue { description: Description::from("Second open task"), state: State::Open, time_created: 0 },
                Issue { description: Description::from("Third open task"), state: State::Open, time_created: 0 },
            ],
            vec![],
            vec![])
    }
}