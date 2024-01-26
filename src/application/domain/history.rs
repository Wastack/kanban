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
pub struct MoveHistoryElements {
    #[serde(default)]
    pub moves: Vec<MoveHistoryElement>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MoveHistoryElement {
    /// Index of the issues that was moved
    pub original_index: usize,

    pub original_state: State,

    /// It can happen that moving changes priorities.
    /// If it does, then new_index is different than original_index.
    pub new_index: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PrioHistoryElement {
    pub original_order: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EditHistoryElement {
    pub original_description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct DeleteHistoryElements {
    #[serde(default)]
    pub deletions: Vec<DeleteHistoryElement>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DeleteHistoryElement {
    /// The position in which the issue had been located just before it was deleted.
    pub(crate) original_position_in_issues: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UndoableHistoryElement {
    Add,
    Delete(DeleteHistoryElements),
    Move(MoveHistoryElements),
    Prio(PrioHistoryElement),
    Edit(EditHistoryElement)
}

