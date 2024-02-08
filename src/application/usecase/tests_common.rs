
#[cfg(test)]
pub(crate) mod tests {
    use std::ops::Deref;
    use crate::application::{Board, Issue};
    use crate::application::issue::{Description};
    use crate::{State};
    use crate::adapters::time_providers::fake::DEFAULT_FAKE_TIME;

    // TODO: move this to a more appropriate place. It's not only a concern for use-cases, but also used for adapters.

    impl Board<Issue> {
        pub(crate) fn assert_issue_count(&self, num: usize) -> &Self {
            assert_eq!(self.entities.len(), num, "Expected board to have {} issues", num);

            self
        }

        pub(crate) fn assert_has_original_issues(&self) -> &Self {
            let original_board = Board::default().with_4_typical_issues();
            assert!(self.entities.len() >= original_board.entities.len(), "Expected board to have the 4 original issues");

            for issue in original_board.entities() {
                let found = self.entities().into_iter().find(
                    |&i| issue.description == i.description
                ).is_some();

                assert!(found, "Expected issue: {:?} to be found in board", issue);
            }

            // TODO: move this assertion to a separate then block
            assert!(self.get_deleted_entities().is_empty(), "Expected not to have deleted issues");
            self
        }

        pub(crate) fn has_the_original_4_issues_in_order(&self) -> &Self {
            typical_4_issues()
                .into_iter()
                .rev()
                .zip(self.entities()
                    .iter()
                    .map(|x|x.deref()))
                .for_each(|(expected, actual)|{
                assert_eq!(actual, &expected, "Expected Issue to be the original one");
            });

            self
        }

        pub(crate) fn with_4_typical_issues(mut self) -> Self {
            typical_4_issues().into_iter().for_each(|i|self.append_entity(i));
            self
        }

        pub(crate) fn with_issue(mut self, issue: Issue) -> Self {
            self.append_entity(issue);
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
                time_created: DEFAULT_FAKE_TIME,
            },
        ]
    }

}

