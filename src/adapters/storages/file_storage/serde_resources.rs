use serde::{Deserialize, Serialize};
use std::ops::Deref;
use crate::application::{Board, Issue, State};
use crate::application::domain::history::{DeleteHistoryElement, DeleteHistoryElements, EditHistoryElement, MoveHistoryElement, MoveHistoryElements, PrioHistoryElement, UndoableHistoryElement};
use crate::application::issue::{Description, Entity};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StoredBoard {
    #[serde(default)]
    issues: Vec<StoredIssue>,

    #[serde(default)]
    deleted_issues: Vec<StoredIssue>,

    #[serde(default)]
    history: Vec<StoredUndoableHistoryElement>,
}

impl From<&Board<Issue>> for StoredBoard {
    fn from(b: &Board<Issue>) -> Self {
        Self {
            issues: b.entities().into_iter().map(|e| StoredIssue::from(e.deref())).collect(),
            deleted_issues: b.get_deleted_entities().into_iter().map(|e| StoredIssue::from(e.deref())).collect(),
            history: b.history().iter().map(|x| x.into()).collect(),
        }
    }
}

impl Into<Board<Issue>> for StoredBoard {
    fn into(self) -> Board<Issue> {
        Board::new(
            self.issues.into_iter().map(|x| x.into()).collect(),
            self.deleted_issues.into_iter().map(|x| x.into()).collect(),
            self.history.into_iter().map(|x| x.into()).collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoredIssue {
    /// Description (content) of the ticket
    description: String,
    /// State of the ticket
    state: StoredState,
    /// Time in seconds since the issue was created
    time_created: u64,
}

impl Into<Issue> for StoredIssue {
    fn into(self) -> Issue {
        Issue {
            description: Description::from(self.description.as_str()),
            state: self.state.into(),
            time_created: self.time_created,
        }
    }
}

impl From<&Issue> for StoredIssue {
    fn from(issue: &Issue) -> Self {
        Self {
            description: issue.description.to_string(),
            state: issue.state.into(),
            time_created: issue.time_created,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StoredMoveHistoryElements {
    pub moves: Vec<StoredMoveHistoryElement>,
}

impl From<&MoveHistoryElements> for StoredMoveHistoryElements {
    fn from(value: &MoveHistoryElements) -> Self {
        Self {
            moves: value.moves.iter().map(|x| x.into()).collect(),
        }
    }
}

impl Into<MoveHistoryElements> for StoredMoveHistoryElements {
    fn into(self) -> MoveHistoryElements {
        MoveHistoryElements {
            moves: self.moves.into_iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StoredMoveHistoryElement {
    /// Index of the issues that was moved
    pub original_index: usize,

    pub original_state: StoredState,

    /// It can happen that moving changes priorities.
    /// If it does, then new_index is different from original_index.
    pub new_index: usize,
}

impl From<&MoveHistoryElement> for StoredMoveHistoryElement {
    fn from(value: &MoveHistoryElement) -> Self {
        Self {
            original_index: value.original_index,
            original_state: value.original_state.into(),
            new_index: value.new_index,
        }
    }
}

impl Into<MoveHistoryElement> for StoredMoveHistoryElement {
    fn into(self) -> MoveHistoryElement {
        MoveHistoryElement {
            original_index: self.original_index,
            original_state: self.original_state.into(),
            new_index: self.new_index,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StoredPrioHistoryElement {
    pub original_order: usize,
}
impl From<&PrioHistoryElement> for StoredPrioHistoryElement {
    fn from(value: &PrioHistoryElement) -> Self {
        Self {
            original_order: value.original_order,
        }
    }
}

impl Into<PrioHistoryElement> for StoredPrioHistoryElement {
    fn into(self) -> PrioHistoryElement {
        PrioHistoryElement {
            original_order: self.original_order,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StoredEditHistoryElement {
    pub original_description: String,
    pub index: usize ,
}

impl From<&EditHistoryElement> for StoredEditHistoryElement {
    fn from(value: &EditHistoryElement) -> Self {
        Self {
            original_description: value.original_description.clone(),
            index: value.index,
        }
    }
}

impl Into<EditHistoryElement> for StoredEditHistoryElement {
    fn into(self) -> EditHistoryElement {
        EditHistoryElement {
            original_description: self.original_description.clone(),
            index: self.index,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct StoredDeleteHistoryElements {
    pub deletions: Vec<StoredDeleteHistoryElement>,
}

impl From<&DeleteHistoryElements> for StoredDeleteHistoryElements {
    fn from(value: &DeleteHistoryElements) -> Self {
        Self {
            deletions: value.deletions.iter().map(|x| x.into()).collect(),
        }
    }
}

impl Into<DeleteHistoryElements> for StoredDeleteHistoryElements {
    fn into(self) -> DeleteHistoryElements {
        DeleteHistoryElements {
            deletions: self.deletions.into_iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StoredDeleteHistoryElement {
    /// The position in which the issue had been located just before it was deleted.
    pub(crate) original_position_in_issues: usize,
}

impl From<&DeleteHistoryElement> for StoredDeleteHistoryElement {
    fn from(value: &DeleteHistoryElement) -> Self {
        Self {
            original_position_in_issues: value.original_position_in_issues,
        }
    }
}

impl Into<DeleteHistoryElement> for StoredDeleteHistoryElement {
    fn into(self) -> DeleteHistoryElement {
        DeleteHistoryElement {
            original_position_in_issues: self.original_position_in_issues,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum StoredUndoableHistoryElement {
    Add,
    Delete(StoredDeleteHistoryElements),
    Move(StoredMoveHistoryElements),
    Prio(StoredPrioHistoryElement),
    Edit(StoredEditHistoryElement)
}

impl From<&UndoableHistoryElement> for StoredUndoableHistoryElement {
    fn from(e: &UndoableHistoryElement) -> Self {
        match e {
            UndoableHistoryElement::Add => StoredUndoableHistoryElement::Add,
            UndoableHistoryElement::Delete(e) => StoredUndoableHistoryElement::Delete(e.into()),
            UndoableHistoryElement::Move(e) => StoredUndoableHistoryElement::Move(e.into()),
            UndoableHistoryElement::Prio(e) => StoredUndoableHistoryElement::Prio(e.into()),
            UndoableHistoryElement::Edit(e) => StoredUndoableHistoryElement::Edit(e.into()),
        }
    }
}

impl Into<UndoableHistoryElement> for StoredUndoableHistoryElement {
    fn into(self) -> UndoableHistoryElement {
        match self {
            StoredUndoableHistoryElement::Add => UndoableHistoryElement::Add,
            StoredUndoableHistoryElement::Delete(e) => UndoableHistoryElement::Delete(e.into()),
            StoredUndoableHistoryElement::Move(e) => UndoableHistoryElement::Move(e.into()),
            StoredUndoableHistoryElement::Prio(e) => UndoableHistoryElement::Prio(e.into()),
            StoredUndoableHistoryElement::Edit(e) => UndoableHistoryElement::Edit(e.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum StoredState {
    Open,
    Review,
    Done,
}


impl Into<State> for StoredState {
    fn into(self) -> State {
        match self {
            StoredState::Open => State::Open,
            StoredState::Review => State::Review,
            StoredState::Done => State::Done,
        }
    }
}

impl From<State> for StoredState {
    fn from(s: State) -> Self {
        match s {
            State::Open => StoredState::Open,
            State::Review => StoredState::Review,
            State::Done => StoredState::Done,
        }
    }
}