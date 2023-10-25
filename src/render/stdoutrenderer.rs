use crate::model::board::{Board, BoardStateView, IssueRef};
use crate::model::issue::{Described, State, Issue};
use crate::render::render::Renderer;
use colored::Colorize;
use crate::elapsed_time_since_epoch;


#[derive(Default)]
pub struct TabularTextRenderer {}


enum DisplayCategory {
    Normal,
    Overdue,
}

fn categorize(issue: &Issue) -> DisplayCategory {
    let now = elapsed_time_since_epoch();
    let two_weeks_in_secs = 60 * 60 * 24 * 14;

    if now - issue.time_created >= two_weeks_in_secs && issue.state == State::Open {
        DisplayCategory::Overdue
    } else {
        DisplayCategory::Normal
    }
}


impl Renderer for TabularTextRenderer {


    fn render_board(&self, board: &Board) -> String {
        let mut issues = board.issues_with_state();

        let mut done_issues_truncated = false;
        let done_issues = issues.get_mut(&State::Done);

        // Keep only the first 4 issues of DONE
        if let Some(done_issues) = done_issues {
            if done_issues.len() > 4 {
                done_issues_truncated = true;
            }
            done_issues.drain(4..);
        }

        vec![
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
                        .get(&tab)
                        .unwrap_or(&Vec::<IssueRef>::default())
                        .iter()

                        // make it to a string with display category (e.g. overdue)
                        .map(|IssueRef {issue, order} |
                                 (
                                     format!("{}: {}", order, issue.description()),
                                     categorize(issue)
                                 )
                        )

                        // apply display category
                        .map(|(text, category) |
                            match category {
                                DisplayCategory::Overdue => text.red().to_string(),
                                DisplayCategory::Normal => text,
                            }
                        )

                        .collect::<Vec<String>>()
                        .join("\n"),

                    // If there are not visible done issue, indicate it with a ...
                    if tab == State::Done && done_issues_truncated {
                        String::from("...")
                    } else {
                        String::default()
                    }
                ].join("\n")
            )
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn state_to_text(state: &State) -> &'static str {
    match state {
        State::Open => "Open",
        State::Review => "Review",
        State::Done => "Done",
    }
}

#[derive(Default)]
pub struct OnlyDoneStdOutRenderer {}

impl Renderer for OnlyDoneStdOutRenderer {
    fn render_board(&self, board: &Board) -> String {
        board.issues_with_state()
            .get(&State::Done)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|IssueRef{issue, order}| format!("{}: {}", order, issue.description()))
            .collect::<Vec<String>>()
            .join("\n")
    }
}