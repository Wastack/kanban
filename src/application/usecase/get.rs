use internal_macros::{PresenterHolder, StorageHolder};
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::application::usecase::usecase::{HasPresenter, HasStorage};

#[derive(Default, StorageHolder, PresenterHolder)]
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
    use std::cell::{RefCell};
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::usecase::get::GetUseCase;
    use crate::application::usecase::test_utils::get_stored_and_presented_board;

    #[test]
    fn test_get_usecase_on_typical_board() {
        // ToDo: this is done somewhere different?
        let mut get_use_case = GetUseCase::<_, NilPresenter> {
            storage: MemoryIssueStorage {
                board: RefCell::new(HistorizedBoard::default().with_4_typical_issues()),
            },

            ..Default::default()
        };

        get_use_case.execute();

        let stored_board = get_stored_and_presented_board(&get_use_case);
        check_boards_are_equal(&stored_board, &HistorizedBoard::default().with_4_typical_issues());
    }
}
