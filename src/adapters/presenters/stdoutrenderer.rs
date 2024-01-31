use crate::application::board::{Board, BoardStateView, IssueRef};
use crate::application::issue::{Described, State};
use crate::application::ports::presenter::Presenter;
use colored::Colorize;
use crate::application::domain::error::DomainError;
use crate::application::domain::issue;
use crate::application::domain::issue::DisplayCategory;

#[derive(Default)]
pub struct TabularTextRenderer {}


impl Presenter for TabularTextRenderer {


    fn render_board(&mut self, board: &Board) {
        let result = Self::format_board(board);

        println!("{}", result)
    }

    fn render_error(&mut self, err: &DomainError) {
        println!("{}", err)
    }
}

impl TabularTextRenderer {
    fn format_board(board: &Board) -> String {
        let mut issues = board.issues_with_state();

        let mut done_issues_truncated = false;

        // Keep only the first 4 issues of DONE
        if let Some(done_issues) = issues.get_mut(&State::Done) {
            if done_issues.len() > 4 {
                done_issues_truncated = true;
            }
            done_issues.drain(4..);
        }

        let result = vec![
            State::Open,
            State::Review,
            State::Done,
        ]
            .into_iter()
            .map(|tab|
                vec![
                    // Header
                    state_to_text(&tab).bold().to_string(),

                    // Display the issues
                    issues
                        // State by state
                        .remove(&tab)
                        .unwrap_or(Vec::<IssueRef>::default())
                        .into_iter()

                        // make it to a string with display category (e.g. overdue)
                        .map(|IssueRef { issue, order }|
                            (
                                format!("{}: {}", order, issue.description()),
                                issue::categorize(issue)
                            )
                        )

                        // apply display category
                        .map(|(text, category)|
                            match category {
                                DisplayCategory::Overdue => text.red().to_string(),
                                DisplayCategory::Normal => text,
                            }
                        )

                        .collect::<Vec<String>>()
                        .join("\n"),

                    // If there are non-visible done issue, indicate it with a ...
                    if tab == State::Done && done_issues_truncated {
                        String::from("...")
                    } else {
                        String::default()
                    }
                ].join("\n")
            )
            .collect::<Vec<String>>()
            .join("\n");
        result
    }
}

fn state_to_text(state: &State) -> &'static str {
    match state {
        State::Open => "Open",
        State::Review => "Review",
        State::Done => "Done",
    }
}


#[cfg(test)]
mod test {
    use std::ops::Deref;
    use crate::adapters::presenters::stdoutrenderer::TabularTextRenderer;
    use crate::application::{Board, Issue, State};
    use crate::application::issue::Description;

    #[test]
    fn test_format_typical_board() {
        // Given a board with some additional done issues
        let board = (0..5).into_iter()
            .fold(Board::default().with_4_typical_issues(), | board, n| board.with_issue(
                Issue::new(Description::from(format!("Done issue number {}", n).deref()), State::Done)
            ));

        // When
        let formatted_board = TabularTextRenderer::format_board(&board);

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
}