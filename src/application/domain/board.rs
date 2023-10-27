use std::collections::HashMap;
use nonempty_collections::NEVec;
use crate::application::issue::Issue;
use serde::{Serialize, Deserialize};
use validated::Validated;
use crate::application::domain::error::{DomainError, DomainResult};
use crate::application::issue::State;



#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub issues: Vec<Issue>,
}

impl Board {
    pub fn get_issue(&self, index: usize) -> DomainResult<&Issue> {
        self.issues.get(index).ok_or(DomainError::new("Index out of range"))
    }

    pub fn get_issue_mut(&mut self, index: usize) -> DomainResult<&mut Issue> {
        self.issues.get_mut(index).ok_or(DomainError::new("Index out of range"))
    }

    pub fn contains(&self, index: usize) -> bool {
        self.issues.len() > index
    }

    pub fn validate_indices(&self, indices: &[usize]) -> Validated<(), DomainError> {
        let errors: Vec<DomainError> = indices
            .iter()
            .filter(|&&i| !self.contains(i))
            .map(|i|DomainError::new(&format!("Index out of range: {}", i)))
            .collect();

        if errors.is_empty() {
            Validated::Good(())
        } else {
            Validated::Fail(NEVec::from_vec(errors).unwrap())
        }
    }
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

        assert!(result.is_fail(), "Expected validation to fail");
    }

    fn given_board_with_2_tasks() -> Board {
        Board {
            issues: vec![
                Issue {
                    description: Description::from("First task"),
                    state: State::Open,
                    time_created: 1698397489,

                },
                Issue {
                    description: Description::from("Second task"),
                    state: State::Review,
                    time_created: 1698397490,

                },
            ],
        }
    }

    #[test]
    fn test_verify_indices_invalid() {
        let board = given_board_with_2_tasks();
        let indices = given_some_indices_are_out_of_range();

        let validated = board.validate_indices(&indices);


        assert!(validated.is_fail(), "Expected validation to fail");
        let Fail(errors) = validated else { panic!() };
        assert_eq!(errors.len(), 2, "Expected 2 errors, but found different number");
        assert_eq!(
            errors[0].description(),
            "Index out of range: 2",
            "Expected specific error message"
        );
        assert_eq!(
            errors[1].description(),
            "Index out of range: 3",
            "Expected specific error message"
        );
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
        assert_eq!(errors.len(), 2, "Expected 2 errors for empty board");
    }

    fn given_empty_board() -> Board {
        Board { issues: vec![] }
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