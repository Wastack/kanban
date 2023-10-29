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

#[cfg(test)]
mod tests {
    use crate::application::{Board, Issue};
    use crate::{IssueStorage, MoveUseCase, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;

    #[test]
    fn test_successful_add_use_case() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![1, 0], &State::Done);

        then_issue_with_index(0, &move_use_case)
            .has_done_state();

        then_issue_with_index(1, &move_use_case)
            .has_done_state();
    }

    fn given_move_use_case_with(board: Board) -> MoveUseCase {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        MoveUseCase {
            storage: Box::new(storage),
            presenter: Box::new(NilPresenter::default()),
        }
    }

    fn then_issue_with_index(index: usize, sut: &MoveUseCase) -> Issue {
        let board = sut.storage.load();

        board.get_issue(index).unwrap().clone()
    }

    impl Issue {
        fn has_done_state(&self) -> &Self {
            assert_eq!(self.state, State::Done, "Expected moved issue to be in done state");
            self
        }
    }

}
