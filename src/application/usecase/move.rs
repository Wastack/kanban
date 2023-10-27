use validated::Validated::Fail;
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
    pub(crate) fn execute(&mut self, indices: &[usize], state: &State) {
        let mut board = self.storage.load();

        let validated = board.validate_indices(indices);

        if let Fail(errors) = validated {
            errors.into_iter()
                .for_each(|e| self.presenter.render_error(&e));
            return

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

        self.storage.save(&board);
        self.presenter.render_board(&board);
    }
}