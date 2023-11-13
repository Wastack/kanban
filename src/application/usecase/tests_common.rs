
#[cfg(test)]
pub(crate) mod tests {
    use crate::application::{Board, Issue};
    use crate::application::issue::{Description};
    use crate::{State};

    impl Board {
        pub(crate) fn did_not_change(&self) -> &Self {
            assert_eq!(self, &Board::default().with_4_typical_issues(), "Expected board not to change");
            self
        }

        pub(crate) fn with_4_typical_issues(mut self) -> Self {
            [
                Issue {
                    description: Description::from("Task inserted first"),
                    state: State::Open,
                    time_created: 1698397489,

                },
                Issue {
                    description: Description::from("Task inserted second"),
                    state: State::Review,
                    time_created: 1698397490,
                },
                Issue {
                    description: Description::from("Task inserted third"),
                    state: State::Done,
                    time_created: 1698397491,
                },
                Issue {
                    description: Description::from("Task inserted fourth"),
                    state: State::Open,
                    time_created: 1698397492,
                },
            ].into_iter().for_each(|i|self.insert_issue(i));

            self
        }
    }
}

