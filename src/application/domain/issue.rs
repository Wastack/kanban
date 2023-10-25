use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Description(pub String);


impl Description {
    pub fn to_str(&self) -> &str {
        return &self.0
    }
}

impl From<&str> for Description {
    fn from(s: &str) -> Self {
        Self(s.trim().to_string())
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


#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    #[serde(default = "elapsed_time_since_epoch")]
    pub(crate) time_created: u64,
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

pub fn elapsed_time_since_epoch() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_secs()
}