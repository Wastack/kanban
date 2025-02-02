use std::cell::LazyCell;
use time::Date;
use crate::application::ports::time::{TodayProvider};

pub(crate) struct SimpleTimeProvider {
    today: LazyCell<Date>
}

impl Default for SimpleTimeProvider {
    fn default() -> Self {
        Self {
            today: LazyCell::new(|| {
                time::OffsetDateTime::now_utc().date()
            })
        }
    }
}

impl TodayProvider for SimpleTimeProvider {
    fn today(&self) -> Date {
        *self.today
    }
}