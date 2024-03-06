use std::fmt::{Display, Formatter};
use std::hash::{Hash};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;
use crate::application::domain::history::Historized;
use crate::application::domain::history::UndoableHistoryElement;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum State {
    Open,
    Review,
    Done,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Description(pub String);

impl From<&str> for Description {
    fn from(s: &str) -> Self {
        Self(s.trim().to_string())
    }
}

impl Description{
    pub fn set(&mut self, new_description: &str) {
        self.0 = new_description.to_string();
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}


impl Display for Description {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Entity<T> {
    /// Uniquely identifies an `Entity` in a `Board`
    pub(crate) id: Uuid,
    pub(crate) content: T,
}

pub trait IdGenerator: Default {
    fn gen(&mut self) -> Uuid;
}


#[derive(Debug, Clone, Default)]
pub struct UUidGenerator;

impl IdGenerator for UUidGenerator {
    fn gen(&mut self) -> Uuid {
        Uuid::new_v4()
    }
}

impl Historized for Issue {
    type HistoryType = UndoableHistoryElement;
}

impl<T> AsRef<T> for Entity<T> {
    fn as_ref(&self) -> &T {
        return &self.content
    }
}

impl<T> Deref for Entity<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T> DerefMut for Entity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}


impl<T> Entity<T> {
    /// This conversion will generate the `id` of the `Entity` by hashing all the fields of the candidate `Entity`.
    pub fn build<IdGen: IdGenerator>(entity: T, id_generator: &mut IdGen) -> Self {
        let id = id_generator.gen();

        Self {
            id,
            content: entity,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Issue {
    /// Description (content) of the ticket
    pub(crate) description: Description,
    /// State of the ticket
    pub(crate) state: State,
    /// Time in seconds since the issue was created
    ///
    /// For backwards compatibility, if the field is missing, we take it as if it was
    /// created just now.
    pub(crate) time_created: u64,
}

impl Issue {

    pub fn category(&self, time_since_epoch: u64) -> IssueCategory {
        let two_weeks_in_secs = 60 * 60 * 24 * 14;

        if time_since_epoch - self.time_created >= two_weeks_in_secs && self.state == State::Open {
            return IssueCategory::Overdue;
        }

        IssueCategory::Normal
    }
}

pub enum IssueCategory {
    Normal,
    Overdue,
}