use time::Date;
use time::macros::date;
use crate::application::ports::time::{CurrentTimeProvider, TodayProvider};

pub(crate) const DEFAULT_FAKE_TIME: u64 = 1706727855;
pub(crate) const DEFAULT_FAKE_TODAY: Date = date!(2025-02-22); // Saturday

// ToDo: migrate to FAKE_TODAY
pub(crate) struct FakeTimeProvider {
    pub(crate) fake_now_answer: u64,
}

impl Default for FakeTimeProvider {
    fn default() -> Self {
        Self {
            fake_now_answer: DEFAULT_FAKE_TIME,
        }
    }
}

impl CurrentTimeProvider for FakeTimeProvider {
    fn now(&self) -> u64 {
        self.fake_now_answer
    }
}

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

