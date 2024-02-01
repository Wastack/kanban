use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Open,
    Review,
    Done,
}

pub trait Stateful {
    fn state(&self) -> &State;
    fn state_mut(&mut self) -> &mut State;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Hash)]
#[serde(rename_all = "camelCase")]
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

pub trait Described {
    fn description(&self) -> &Description;
    fn description_mut(&mut self) -> &mut Description;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity<T: Serialize + Hash> {
    /// Uniquely identifies an `Entity` in a `Board`
    #[serde(skip)]
    pub(crate) id: u64,

    #[serde(flatten)]
    pub(crate) entity: T,
}

impl<T: Serialize + Hash> From<T> for Entity<T> {
    fn from(value: T) -> Self {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        let id = s.finish();
        
        Self {
            id,
            entity: value,
        }
    }
}

impl<T: Serialize + Hash> Deref for Entity<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T: Serialize + Hash> DerefMut for Entity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Hash)]
#[serde(rename_all = "camelCase")]
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

impl Stateful for Issue {
    fn state(&self) -> &State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl Described for Issue {
    fn description(&self) -> &Description {
        &self.description
    }

    fn description_mut(&mut self) -> &mut Description {
        &mut self.description
    }
}

pub enum IssueCategory {
    Normal,
    Overdue,
}