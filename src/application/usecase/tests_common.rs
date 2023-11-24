
#[cfg(test)]
pub(crate) mod tests {
    use crate::application::{Board, DomainResult, Issue};
    use crate::application::issue::{Description};
    use crate::{State};

    impl Board {
        pub(crate) fn has_number_of_issues(&self, num: usize) -> &Self {
            assert_eq!(self.issues_count(), num, "Expected board to have {} issues", num);

            self
        }

        pub(crate) fn has_the_original_4_issues(&self) -> &Self {
            let original_board = Board::default().with_4_typical_issues();
            assert!(self.issues_count() >= original_board.issues_count(), "Expected board to have the 4 original issues");

            for issue in original_board.issues() {
                let found = self.issues().into_iter().find(
                    |&i| issue.description == i.description
                ).is_some();

                assert!(found, "Expected issue: {:?} to be found in board", issue);
            }

            // TODO: move this assertion to a separate then block
            assert!(self.get_deleted_issues().is_empty(), "Expected not to have deleted issues");
            self
        }

        pub(crate) fn has_the_original_4_issues_in_order(&self) -> &Self {
            let issues = typical_4_issues();
            issues.into_iter().rev().zip(self.issues().iter()).for_each(|(expected, actual)|{
                assert_eq!(actual, &expected, "Expected Issue to be the original one");
            });

            self
        }

        pub(crate) fn with_4_typical_issues(mut self) -> Self {
            typical_4_issues().into_iter().for_each(|i|self.add_issue(i));
            self
        }

    }

    fn typical_4_issues() -> [Issue; 4] {
        [
            // index 3
            Issue {
                description: Description::from("Task inserted first"),
                state: State::Open,
                time_created: 1698397489,

            },
            // index 2
            Issue {
                description: Description::from("Task inserted second"),
                state: State::Review,
                time_created: 1698397490,
            },
            // index 1
            Issue {
                description: Description::from("Task inserted third"),
                state: State::Done,
                time_created: 1698397491,
            },
            // index 0
            Issue {
                description: Description::from("Task inserted fourth"),
                state: State::Open,
                time_created: 1698397492,
            },
        ]
    }

    pub(crate) fn then_result<T>(result: &DomainResult<T>) -> DomainResultMatcher<T> {
        DomainResultMatcher {
            result,
        }
    }

    pub(crate) struct DomainResultMatcher<'a, T> {
        result: &'a DomainResult<T>
    }

    impl<T: std::fmt::Debug> DomainResultMatcher<'_, T> {
        pub(crate) fn did_fail(&self) -> &Self {
            assert!(self.result.is_err(), "Expected editing to fail");
            self
        }

        pub(crate) fn did_fail_with_error_message(&self, message: &str) -> &Self {
            assert!(self.result.is_err(), "Expected it to fail");
            assert_eq!(self.result.as_ref().unwrap_err().description(), message, "Expect proper error message");
            self
        }

        pub(crate) fn did_succeed(&self) -> &Self {
            assert!(self.result.is_ok(), "Result failed: {}", self.result.as_ref().unwrap_err().description());
            self
        }
    }

}

