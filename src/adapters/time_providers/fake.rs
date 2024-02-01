use crate::application::ports::time::CurrentTimeProvider;

pub(crate) const DEFAULT_FAKE_TIME: u64 = 1706727855;

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