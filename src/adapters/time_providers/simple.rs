use std::time::{SystemTime, UNIX_EPOCH};
use crate::application::ports::time::CurrentTimeProvider;

#[derive(Default)]
pub(crate) struct SimpleTimeProvider {}

impl CurrentTimeProvider for SimpleTimeProvider {
    fn now(&self) -> u64 {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_epoch.as_secs()
    }
}
