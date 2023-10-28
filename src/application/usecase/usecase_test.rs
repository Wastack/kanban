
#[cfg(test)]
pub(crate) mod tests {
    use crate::application::{Board, Issue};
    use crate::application::issue::{Description};
    use crate::{State};

    pub(crate) fn board_with_4_issues() -> Board {
        Board {
            issues: vec![
                Issue {
                    description: Description::from("First task"),
                    state: State::Open,
                    time_created: 1698397489,

                },
                Issue {
                    description: Description::from("Second task"),
                    state: State::Review,
                    time_created: 1698397490,
                },
                Issue {
                    description: Description::from("Third task"),
                    state: State::Done,
                    time_created: 1698397491,
                },
                Issue {
                    description: Description::from("Forth task"),
                    state: State::Open,
                    time_created: 1698397492,
                },
            ],
        }
    }

    impl Board {
        pub(crate) fn did_not_change(&self) -> &Self {
            assert_eq!(self, &board_with_4_issues(), "Expected board not to change");
            self
        }
    }
}

