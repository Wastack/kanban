use std::cell::LazyCell;
use std::time::{SystemTime, UNIX_EPOCH};
use time::Date;
use crate::application::ports::time::{CurrentTimeProvider, TodayProvider};

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

impl CurrentTimeProvider for SimpleTimeProvider {
    /// ToDo: make current time about today, and not now
    fn now(&self) -> u64 {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_epoch.as_secs()
    }
}

impl TodayProvider for SimpleTimeProvider {
    fn today(&self) -> Date {
        *self.today
    }
}