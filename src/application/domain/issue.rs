use std::fmt::{Display, Formatter};
use std::hash::{Hash};
use std::ops::{Deref, DerefMut};
use time::Duration;
use uuid::Uuid;
use crate::application::domain::history::Historized;
use crate::application::domain::history::UndoableHistoryElement;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum State {
    Open,
    Review,
    Done,
}

impl Default for State {
    fn default() -> Self {
        Self::Open
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Default)]
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
    pub(crate) time_created: time::Date,

    /// Due date of an issue
    pub(crate) due_date: Option<time::Date>,
}

impl Issue {

    pub fn category(&self, today: time::Date) -> IssueCategory {
        let time_since_creation = today - self.time_created;
        let time_since_due = self.due_date.map(|due_date| due_date - today);

        if time_since_creation > Duration::days(13)
                || time_since_due.is_some_and(|d| d < Duration::default()) {
            IssueCategory::Overdue
        } else if time_since_due.is_some_and(|d| d == Duration::default()) {
            IssueCategory::DueToday
        } else {
            IssueCategory::Normal
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum IssueCategory {
    Normal,
    Overdue,
    DueToday
}

#[cfg(test)]
mod tests {
    use assert2::{check};
    use time::Date;
    use time::macros::date;
    use super::*;

    #[test]
    fn test_category_normal() {
        let today = date!(2021 - 9 - 9);

        for (actual_creation_date, actual_due_date, expected_category) in
            [
                (date!(2021 - 9 - 8), Some(date!(2021 - 9 - 11)), IssueCategory::Normal),
                (date!(2021 - 8 - 25), None, IssueCategory::Overdue),
                (date!(2021 - 9 - 8), Some(date!(2021 - 9 - 9)), IssueCategory::DueToday),
                (date!(2021 - 9 - 8), Some(date!(2021 - 9 - 8)), IssueCategory::Overdue),
            ] {
            let issue = given_issue_with(actual_creation_date, actual_due_date);

            let category = issue.category(today);
            check!(category == expected_category,
                "creation_date = {:?}, due_date = {:?}", actual_creation_date, actual_due_date);
        }
    }

    fn given_issue_with(time_created: Date, due_date: Option<Date>) -> Issue {
        let issue = Issue {
            description: Description::from("an issue"),
            state: State::Open,
            time_created,
            due_date,
        };
        issue
    }
}