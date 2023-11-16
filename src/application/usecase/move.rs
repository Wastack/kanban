use validated::Validated;
use validated::Validated::Fail;
use crate::application::domain::error::DomainError;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::State;


#[derive(Default)]
pub(crate) struct MoveUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl MoveUseCase {
    pub(crate) fn execute(&mut self, indices: &[usize], state: State) -> Validated<(), DomainError> {
        let mut board = self.storage.load();

        let validated = board.validate_indices(indices);

        if let Fail(errors) = &validated {
            errors.into_iter()
                .for_each(|e| self.presenter.render_error(&e));

            return validated;
        }

        // TODO implement history

        for &index in indices {
            board.move_issue(index, state).unwrap();
        }


        self.storage.save(&board);
        self.presenter.render_board(&board);

        Validated::Good(())
    }
}

#[cfg(test)]
mod tests {
    use validated::Validated;
    use validated::Validated::Fail;
    use crate::application::{Board, Issue};
    use crate::{IssueStorage, MoveUseCase, State};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::error::{DomainError};

    #[test]
    fn test_successful_add_use_case() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        move_use_case.execute(&vec![1, 0], State::Done);

        then_issue_with_index(0, &move_use_case)
            .has_done_state();

        then_issue_with_index(1, &move_use_case)
            .has_done_state();
    }

    #[test]
    fn test_indices_out_of_range() {
        let mut move_use_case = given_move_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        let result = move_use_case.execute(&vec![1, 4, 5], State::Done);

        then_moving(&result)
            .did_fail()
            .did_produce_two_errors();

        then_stored_board(&move_use_case)
            .has_number_of_issues(4)
            .has_the_original_4_issues();
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

    fn then_stored_board(u: &MoveUseCase) -> Board {
        u.storage.load()
    }

    fn then_moving(result: &Validated<(), DomainError>) -> MovingResult {
        MovingResult {
            result,
        }
    }

    struct MovingResult<'a> {
        result: &'a Validated<(), DomainError>
    }

    impl MovingResult<'_> {
        fn did_fail(&self) -> &Self {
            assert!(self.result.is_fail(), "Expected deletion to fail");
            self
        }

        fn did_produce_two_errors(&self) -> &Self {
            let Fail(errors) = self.result else { panic!("Expected moving to fail") };
            assert_eq!(errors.len(), 2, "Expected to produce 2 errors");
            assert_eq!(errors[0].description(), "Index out of range: 4", "Expected specific error message");
            assert_eq!(errors[1].description(), "Index out of range: 5", "Expected specific error message");
            self
        }
    }

    impl Issue {
        fn has_done_state(&self) -> &Self {
            assert_eq!(self.state, State::Done, "Expected moved issue to be in done state");
            self
        }
    }

}
