use crate::application::ports::time::CurrentTimeProvider;

pub(crate) struct FakeTimeProvider {
    pub(crate) fake_now_answer: u64,
}

impl Default for FakeTimeProvider {
    fn default() -> Self {
        Self {
            fake_now_answer: 1706727855,
        }
    }
}

impl CurrentTimeProvider for FakeTimeProvider {
    fn now(&self) -> u64 {
        self.fake_now_answer
    }
}