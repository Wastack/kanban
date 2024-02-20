use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct GetUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P
}

impl<I: IssueStorage, P: Presenter> GetUseCase<I, P> {
    pub(crate) fn execute(&mut self) {
        let board = self.storage.load();
        self.presenter.render_board(&board);
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::{Board};
    use crate::application::usecase::get::GetUseCase;
    use crate::application::usecase::tests_common::tests::check_boards_are_equal;

    #[test]
    fn test_get_usecase_on_typical_board() {
        let mut get_use_case = GetUseCase::<_, NilPresenter> {
            storage: MemoryIssueStorage {
                board: Board::default().with_4_typical_issues()
            },

            ..Default::default()
        };

        get_use_case.execute();

        let presented_board = get_use_case.presenter.last_board_rendered.expect("Expected a board to be presented");
        check_boards_are_equal(&presented_board, &Board::default().with_4_typical_issues());
    }
}
