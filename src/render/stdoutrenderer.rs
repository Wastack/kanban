use crate::model::board::Board;
use crate::model::issue::{Described, Stateful, State, Issue};
use crate::render::render::Renderer;
use colored::Colorize;


#[derive(Default)]
pub struct TabularTextRenderer {}


impl Renderer for TabularTextRenderer {

    /// An example output:
    ///
    /// <b>Open</b>
    /// 0: foo
    /// 5: bar
    ///
    /// <b>In Progress</b>
    /// 1: baz
    /// 3: and so on
    fn render_board(&self, board: &Board) -> String {
        vec![
            State::Analysis,
            State::Open,
            State::InProgress,
            State::Review,
            State::Done,
        ].into_iter().map(|tab|
            vec![
                state_to_text(&tab).bold().to_string(),
                board.issues
                    .iter()
                    .enumerate()
                    .filter(|(_, issue)|  *issue.state() == tab)
                    .fold::<String, _>(String::new(),|current: String, (index, issue): (usize, &Issue) |
                        current + &format!("{}: {}\n", index, issue.description()))
            ].join("\n")
        )
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn state_to_text(state: &State) -> &'static str {
    match state {
        State::Analysis => "Analysis",
        State::Open => "Open",
        State::InProgress => "In Progress",
        State::Review => "Review",
        State::Done => "Done",
        _ => panic!("unknown state")
    }
}