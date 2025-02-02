pub(crate) trait CurrentTimeProvider {
    /// Fetched current system time
    fn now(&self) -> u64;
}

pub(crate) trait TodayProvider {
    fn today(&self) -> time::Date;
}