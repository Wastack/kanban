use std::fmt::{Display, Formatter};
use std::hash::{Hash};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;
use crate::application::board::Historized;
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
pub struct Entity<T, IdGen: IdGenerator = UUidGenerator> {
    /// Uniquely identifies an `Entity` in a `Board`
    pub(crate) id: Uuid,
    pub(crate) entity: T,

    id_generator_type: PhantomData<IdGen>
}

pub trait IdGenerator {
    fn gen() -> Uuid;
}


#[derive(Debug, Clone)]
pub struct UUidGenerator;

impl IdGenerator for UUidGenerator {
    fn gen() -> Uuid {
        Uuid::new_v4()
    }
}

impl Historized for Issue {
    type HistoryType = UndoableHistoryElement;
}

impl<T, IdGen: IdGenerator> AsRef<T> for Entity<T, IdGen> {
    fn as_ref(&self) -> &T {
        return &self.entity
    }
}

impl<T, IdGen: IdGenerator> From<T> for Entity<T, IdGen> {
    /// This conversion will generate the `id` of the `Entity` by hashing all the fields of the candidate `Entity`.
    fn from(value: T) -> Self {
        let id = IdGen::gen();

        Self {
            id,
            entity: value,

            id_generator_type: Default::default(),
        }
    }
}

impl<T, IdGen: IdGenerator> Deref for Entity<T, IdGen> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T, IdGen: IdGenerator> DerefMut for Entity<T, IdGen> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
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