use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Analysis,
    Blocked,
    Open,
    InProgress,
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
    pub(crate) description: Description,
    pub(crate) state: State,
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