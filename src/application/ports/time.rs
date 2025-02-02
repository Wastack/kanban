pub(crate) trait TodayProvider {
    fn today(&self) -> time::Date;
}