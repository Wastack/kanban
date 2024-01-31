pub(crate) trait CurrentTimeProvider {
    /// Fetched current system time
    fn now(&self) -> u64;
}