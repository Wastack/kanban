use crate::State;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct History {
    pub(crate) elements: Vec<UndoableHistoryElement>,
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


#[cfg(test)]
pub(crate) mod test {
    use crate::application::domain::history::{History, UndoableHistoryElement};

    impl History {
        pub(crate) fn with_element(mut self, e: UndoableHistoryElement) -> Self {
            self.elements.push(e);
            self
        }
    }
}