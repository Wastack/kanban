
#[cfg(test)]
pub(crate) mod tests {
    use std::ops::Deref;
    use assert2::check;
    use crate::application::{Board, Issue};
    use crate::application::issue::{Description, Entity};
    use crate::{State};
    use crate::adapters::time_providers::fake::DEFAULT_FAKE_TIME;

    // TODO: move this to a more appropriate place. It's not only a concern for use-cases, but also used for adapters.

    impl Board<Issue> {
        pub(crate) fn assert_issue_count(&self, num: usize) -> &Self {
            assert_eq!(self.entity_count(), num, "Expected board to have {} issues", num);

            self
        }

        pub(crate) fn assert_has_original_issues(&self) -> &Self {
            let original_board = Board::default().with_4_typical_issues();
            check!(self.entity_count() >= original_board.entity_count(), "Expected board to have the 4 original issues");

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
            // todo: don't do it with append
            typical_4_issues().into_iter().for_each(|i|self.append_entity(i));
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

    pub(crate) fn check_compare_issues(actual: &[Entity<Issue>], expected: &[Entity<Issue>]) {
        actual.into_iter().map(|e| e)
            .zip(expected.into_iter())
            .for_each(|(actual, expected)| {
                check!(actual.as_ref() == expected.as_ref());
            });
    }

    pub(crate) fn check_boards_are_equal(actual: &Board<Issue>, expected: &Board<Issue>) {
        check_compare_issues(actual.entities(), expected.entities());
        check_compare_issues(actual.get_deleted_entities(), expected.get_deleted_entities());
        check!(actual.history() == expected.history(), "Expected board to have the same history");

    }
}

