use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::State;
use crate::application::domain::issue::Stateful;


#[derive(Default)]
pub(crate) struct MoveUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl MoveUseCase {
    pub(crate) fn execute(&self, indices: &[usize], state: &State) {
        let mut board = self.storage.load();

        // Check if all indices are valid
        if !indices.iter().all(|i|*i < board.issues.len()) {
            if indices.len() > 1 {
                panic!("at least one of the indices specified are out of range")
            } else {
                panic!("index out of range")
            }
        }

        for index in indices {
            let current_state = board.issues[*index].state_mut();

            if current_state != state{
                *current_state = *state;

                if *state == State::Done {
                    board.prio_top_in_category(*index);
                }
            }
        }

        self.presenter.render_board(&board);
    }
}