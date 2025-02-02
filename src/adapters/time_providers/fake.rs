use time::Date;
use time::macros::date;
use crate::application::ports::time::{TodayProvider};

pub(crate) const DEFAULT_FAKE_TODAY: Date = date!(2025-02-22); // Saturday

pub struct FakeTodayProvider {
    pub(crate) fake_today_answer: Date,
}

impl Default for FakeTodayProvider {
    fn default() -> Self {
        Self {
            fake_today_answer: DEFAULT_FAKE_TODAY,
        }
    }
}

impl TodayProvider for FakeTodayProvider {
    fn today(&self) -> Date {
        self.fake_today_answer
    }
}

