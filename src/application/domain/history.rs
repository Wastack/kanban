use serde::{Serialize, Deserialize};
use crate::State;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct History {
    #[serde(default)]
    elements: Vec<UndoableHistoryElement>,
}

impl History {
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn all(&self) -> &[UndoableHistoryElement] {
        &self.elements
    }

    pub fn push(&mut self, element: UndoableHistoryElement) {
        self.elements.push(element);
    }

    pub fn peek(&self) -> Option<&UndoableHistoryElement> {
        self.elements.last()
    }

    pub fn pop(&mut self) -> Option<UndoableHistoryElement> {
        self.elements.pop()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MoveHistoryElement {
    // TODO this is not enough to undo deletion
    pub original_state: State,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PrioHistoryElement {
    pub original_order: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EditHistoryElement {
    pub original_description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DeleteHistoryElements {
    // TODO this is not enough to undo deletion. Original position in array is needed for each
    pub number_of_issues_deleted: usize,

    pub deletions: Vec<DeleteHistoryElement>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DeleteHistoryElement {
    /// The position in which the issue had been located just before it was deleted.
    original_position_in_issues: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UndoableHistoryElement {
    Add,
    Delete(DeleteHistoryElements),
    Move(MoveHistoryElement),
    Prio(PrioHistoryElement),
    Edit(EditHistoryElement)
}

