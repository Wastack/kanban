use crate::application::board::{Board, BoardStateView, IssueRef};
use crate::application::issue::{State};
use crate::application::ports::presenter::Presenter;
use colored::{ColoredString, Colorize};
use crate::adapters::presenters::stdoutrenderer::MaybeFormattedString::{Formatted, NonFormatted};
use crate::application::domain::error::DomainError;
use crate::application::domain::issue::IssueCategory;
use crate::application::Issue;
use crate::application::ports::time::CurrentTimeProvider;
use itertools::Itertools;

#[derive(Default)]
pub(crate) struct TabularTextRenderer<T: CurrentTimeProvider> {
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


impl<T: CurrentTimeProvider> Presenter for TabularTextRenderer<T> {


    fn render_board(&mut self, board: &Board<Issue>) {
        let result = self.format_board(board);

        println!("{}", result)
    }

    fn render_error(&mut self, err: &DomainError) {
        println!("{}", err)
    }
}

impl<T: CurrentTimeProvider> TabularTextRenderer<T> {
    fn format_board(&self, board: &Board<Issue>) -> String {
        self.build_formatted_text_chunks(board)
            .into_iter()
            .map(|t| t.to_string())
            .join("\n")
    }

    fn build_formatted_text_chunks<'a>(&'a self, board: &'a Board<Issue>) -> impl Iterator<Item = MaybeFormattedString> + 'a  {
        let mut issues = board.issues_with_state();

        let mut done_issues_truncated = false;

        // Keep only the first 4 issues of DONE
        if let Some(done_issues) = issues.get_mut(&State::Done) {
            if done_issues.len() > 4 {
                done_issues_truncated = true;
            }
            done_issues.drain(4..);
        }

        let current_time = self.time_provider.now();

        vec![
            State::Open,
            State::Review,
            State::Done,
        ]
            .into_iter()
            .map(move |tab| {
                let current_time = current_time; // capture to force closure to be FnOnce
                vec![
                    // Header
                    Formatted((match &tab {
                        State::Open => "Open",
                        State::Review => "Review",
                        State::Done => "Done",
                    }).bold()),
                ].into_iter().chain(
                    // Display the issues
                    issues
                        // State by state
                        .remove(&tab)
                        .unwrap_or(Vec::<IssueRef>::default())
                        .into_iter()

                        // make it to a string with display category (e.g. overdue)
                        .map(move |IssueRef { issue, order }|
                            {
                                (
                                    format!("{}: {}", order, issue.description),
                                    issue.category(current_time)
                                )
                            }
                        )

                        // apply display category
                        .map(|(text, category)|
                            match category {
                                IssueCategory::Overdue => Formatted(text.red()),
                                IssueCategory::Normal => NonFormatted(text),
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
    use crate::adapters::presenters::stdoutrenderer::{TabularTextRenderer};
    use crate::adapters::time_providers::fake::FakeTimeProvider;
    use crate::application::{Board, Issue, State};
    use crate::application::issue::Description;
    use assert2::{check};
    use colored::{Colorize};
    use crate::adapters::presenters::stdoutrenderer::MaybeFormattedString::{Formatted, NonFormatted};

    #[test]
    fn test_format_empty_board() {
        let formatted_board = TabularTextRenderer::<FakeTimeProvider>::default()
            .format_board(&Board::default());

        check!(formatted_board == "Open\n\nReview\n\nDone\n");
    }

    #[test]
    fn test_format_typical_board() {
        let board = given_board();

        // When
        let formatted_board = TabularTextRenderer::<FakeTimeProvider>::default().format_board(&board);

        // Then
        assert_eq!(formatted_board, r#"Open
5: Task inserted fourth
8: Task inserted first

Review
7: Task inserted second

Done
0: Done issue number 4
1: Done issue number 3
2: Done issue number 2
3: Done issue number 1
..."#);
    }

    #[test]
    fn test_formatted_text_chunks() {
        let board = given_board();
        let text_renderer = TabularTextRenderer::<FakeTimeProvider>::default();

        let mut formatted_chunks = text_renderer.build_formatted_text_chunks(&board);

        [
            Formatted("Open".bold()),
            NonFormatted(String::from("5: Task inserted fourth")),
            Formatted("8: Task inserted first".red()),
            NonFormatted(String::default()), // new line
            Formatted("Review".bold()),
            NonFormatted(String::from("7: Task inserted second")),
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

    fn given_board() -> Board<Issue> {
        let board = (0..5).into_iter()
            // Let's give it some additional done issues, so that we can test that `...` appears at the end
            .fold(Board::default().with_4_typical_issues(), |board, n| board.with_issue(
                Issue {
                    description: Description::from(format!("Done issue number {}", n).deref()),
                    state: State::Done,
                    time_created: 0,
                }
            ));
        board
    }

}