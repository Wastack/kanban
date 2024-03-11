use crate::application::State;

#[derive(Clone, Debug, PartialEq)]
pub struct MoveHistoryElements {
    pub moves: Vec<MoveHistoryElement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MoveHistoryElement {
    /// Index of the issues that was moved
    pub original_index: usize,

    pub original_state: State,

    /// It can happen that moving changes priorities.
    /// If it does, then new_index is different from original_index.
    pub new_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrioHistoryElement {
    pub original_order: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditHistoryElement {
    pub original_description: String,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteHistoryElements {
    pub deletions: Vec<DeleteHistoryElement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteHistoryElement {
    /// The position in which the issue had been located just before it was deleted.
    pub(crate) original_position_in_issues: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UndoableHistoryElement {
    Add,
    Delete(DeleteHistoryElements),
    Move(MoveHistoryElements),
    Prio(PrioHistoryElement),
    Edit(EditHistoryElement)
}


#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct History<H> {
    /// The top (last) element denotes the most recently performed action.
    pub stack: Vec<H>
}

impl<H> Default for History<H> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}

/// Defines what is the type that is used to define history elements in the board. Actions on the type becomes undo-able.
pub trait Historized {
    type HistoryType;
}

impl<H> History<H> {
    pub fn add(&mut self, elem: H) {
        self.stack.push(elem)
    }

    pub fn last(&self) -> Option<&H> {
        self.stack.last()
    }

    pub fn pop(&mut self) -> Option<H> {
        self.stack.pop()
    }
}
