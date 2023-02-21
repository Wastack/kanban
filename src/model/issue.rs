use std::fmt::{Display, Formatter, write};

#[derive(Clone)]
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

pub struct Description(String);

impl Display for Description {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, self.0)
    }
}

pub trait Described {
    fn description(&self) -> &Description;
    fn description_mut(&mut self) -> &mut Description;
}

pub struct Issue {
    description: Description,
    state: State,
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