use std::collections::HashMap;
use crate::application::issue::State;
use crate::application::ports::presenter::Presenter;
use colored::{ColoredString, Colorize};
use crate::adapters::presenters::stdoutrenderer::MaybeFormattedString::{Formatted, NonFormatted};
use crate::application::domain::error::DomainError;
use crate::application::domain::issue::IssueCategory;
use crate::application::Issue;
use crate::application::ports::time::{TodayProvider};
use itertools::Itertools;
use crate::application::domain::historized_board::HistorizedBoard;

#[derive(Default)]
pub(crate) struct TabularTextRenderer<T: TodayProvider> {
    time_provider: T,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum MaybeFormattedString {
    NonFormatted(String),
    Formatted(ColoredString)
}

impl MaybeFormattedString {
    fn to_string(self) -> String {
        match self {
            NonFormatted(t) => t,
            Formatted(t) => t.to_string(),
        }
    }
}


impl<T: TodayProvider> Presenter for TabularTextRenderer<T> {


    fn render_board(&self, board: &HistorizedBoard<Issue>) {
        let result = self.format_board(board);

        println!("{}", result)
    }

    fn render_error(&self, err: &DomainError) {
        println!("{}", err)
    }
}

impl<T: TodayProvider> TabularTextRenderer<T> {
    fn format_board(&self, board: &HistorizedBoard<Issue>) -> String {
        self.build_formatted_text_chunks(board)
            .into_iter()
            .map(|t| t.to_string())
            .join("\n")
    }

    fn build_formatted_text_chunks<'a>(&'a self, board: &'a HistorizedBoard<Issue>) -> impl Iterator<Item = MaybeFormattedString> + 'a  {
        let mut issues_categorised_by_state = board.entities().iter()
            .enumerate()
            .map(|(index, issue) | (issue.state, (index, issue)))
            .fold(HashMap::new(), |mut acc, (state, issue_ref) | {
                acc.entry(state).or_insert_with(Vec::new).push(issue_ref);
                acc
            });

        let mut done_issues_truncated = false;

        // Keep only the first 4 issues of DONE
        if let Some(done_issues) = issues_categorised_by_state.get_mut(&State::Done) {
            if done_issues.len() > 4 {
                done_issues_truncated = true;
                done_issues.drain(4..);
            }
        }

        let today = self.time_provider.today();

        vec![
            State::Open,
            State::Review,
            State::Done,
        ]
            .into_iter()
            .map(move |tab| {
                let today = today; // capture to force closure to be FnOnce
                vec![
                    // Header
                    Formatted((match &tab {
                        State::Open => "Open",
                        State::Review => "Review",
                        State::Done => "Done",
                    }).bold()),
                ].into_iter().chain(
                    // Display the issues
                    issues_categorised_by_state
                        // State by state
                        .remove(&tab)
                        .unwrap_or(Vec::default())
                        .into_iter()

                        // make it to a string with display category (e.g. overdue)
                        .map(move | (index, issue)|
                            {
                                (
                                    if let Some(due) = &issue.due_date {
                                        format!("{}: {}\t{}", index, issue.description, due)
                                    } else {
                                        format!("{}: {}", index, issue.description)
                                    },
                                    issue.category(today)
                                )
                            }
                        )

                        // apply display category
                        .map(|(text, category)|
                            match category {
                                IssueCategory::Overdue => Formatted(text.red()),
                                IssueCategory::Normal => NonFormatted(text),
                                IssueCategory::DueToday => Formatted(text.yellow()),
                            }
                        )

                ).chain(
                    std::iter::once(
                        NonFormatted(
                            if tab == State::Done && done_issues_truncated {
                                String::from("...")
                            } else {
                                String::default()
                            }
                        )
                    )
                )
            }).flatten()
    }
}


#[cfg(test)]
mod test {
    use std::ops::Deref;
    use crate::adapters::presenters::stdoutrenderer::TabularTextRenderer;
    use crate::adapters::time_providers::fake::{FakeTodayProvider, DEFAULT_FAKE_TODAY};
    use crate::application::{Issue, State};
    use crate::application::issue::Description;
    use assert2::check;
    use colored::Colorize;
    use crate::adapters::presenters::stdoutrenderer::MaybeFormattedString::{Formatted, NonFormatted};
    use crate::application::domain::historized_board::HistorizedBoard;
    use time::macros::date;

    #[test]
    fn test_format_empty_board() {
        let board = HistorizedBoard::default();
        let text_renderer = TabularTextRenderer::<FakeTodayProvider>::default();

        let mut formatted_chunks = text_renderer.build_formatted_text_chunks(&board);
        [
            Formatted("Open".bold()),
            NonFormatted(String::default()),
            Formatted("Review".bold()),
            NonFormatted(String::default()),
            Formatted("Done".bold()),
            NonFormatted(String::default()),
        ].into_iter().for_each(|expected| {
            let chunk = formatted_chunks.next().expect("Expected more chunks of formatted output");
            check!(chunk == expected);
        });

        check!(formatted_chunks.next() == None, "Expected not to have any more formatted output");

    }

    #[test]
    fn test_format_1_done_board() {
        let board = HistorizedBoard::default()
            .with_issue(Issue {
                description: Description::from("An issue in done"),
                state: State::Done,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            });
        let text_renderer = TabularTextRenderer::<FakeTodayProvider>::default();

        let mut formatted_chunks = text_renderer.build_formatted_text_chunks(&board);
        [
            Formatted("Open".bold()),
            NonFormatted(String::default()),
            Formatted("Review".bold()),
            NonFormatted(String::default()),
            Formatted("Done".bold()),
            NonFormatted(String::from("0: An issue in done")),
            NonFormatted(String::default()),
        ].into_iter().for_each(|expected| {
            let chunk = formatted_chunks.next().expect("Expected more chunks of formatted output");
            check!(chunk == expected);
        });

        check!(formatted_chunks.next() == None, "Expected not to have any more formatted output");
    }

    #[test]
    fn test_formatted_text_chunks() {
        let board = given_board();
        let text_renderer = TabularTextRenderer::<FakeTodayProvider>::default();

        let mut formatted_chunks = text_renderer.build_formatted_text_chunks(&board);

        [
            Formatted("Open".bold()),
            Formatted("5: An open issue overdue".red()),
            NonFormatted(String::from("6: An open issue not overdue\t2025-03-24")),
            NonFormatted(String::default()), // new line
            Formatted("Review".bold()),
            NonFormatted(String::from("7: An issue in review")),
            NonFormatted(String::default()),
            Formatted("Done".bold()),
            NonFormatted(String::from("0: Done issue number 4")),
            NonFormatted(String::from("1: Done issue number 3")),
            NonFormatted(String::from("2: Done issue number 2")),
            NonFormatted(String::from("3: Done issue number 1")),
            NonFormatted(String::from("...")),
        ].into_iter().for_each(|expected| {
            let chunk = formatted_chunks.next().expect("Expected more chunks of formatted output");
            check!(chunk == expected);
        });

        check!(formatted_chunks.next() == None, "Expected not to have any more formatted output");
    }

    fn given_board() -> HistorizedBoard<Issue> {
        let board = HistorizedBoard::new(
            (0..5).into_iter().rev().map(|n| Issue {
                description: Description::from(format!("Done issue number {}", n).deref()),
                state: State::Done,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None
            })
                .chain(
                    vec![
                        Issue {
                            description: Description::from("An open issue overdue"),
                            state: State::Open,
                            time_created: date!(2024-02-02),
                            due_date: None,
                        },
                        Issue {
                            description: Description::from("An open issue not overdue"),
                            state: State::Open,
                            time_created: date!(2025-02-09),
                            due_date: Some(date!(2025-03-24)),
                        },
                        Issue {
                            description: Description::from("An issue in review"),
                            state: State::Review,
                            time_created: DEFAULT_FAKE_TODAY,
                            due_date: None,
                        },

                    ].into_iter()
                )
                .collect()
            , vec![], vec![]);


        board // the additional 4 issues as ususal
    }

}