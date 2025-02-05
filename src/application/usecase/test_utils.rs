use crate::adapters::presenters::nil_presenter::test::NilPresenter;
use crate::adapters::storages::IssueStorage;
use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
use crate::application::board::test_utils::check_boards_are_equal;
use crate::application::domain::historized_board::HistorizedBoard;
use crate::application::Issue;
use crate::application::usecase::usecase::{HasStorage, HasPresenter};

pub fn get_stored_and_presented_board<T: HasPresenter<NilPresenter> + HasStorage<MemoryIssueStorage>>(use_case: &T) -> HistorizedBoard<Issue> {
    let stored_board = use_case.storage_ref().load();
    let presented_board = use_case.presenter_ref().last_board_rendered.as_ref().expect("board to be presented");
    check_boards_are_equal(&presented_board, &stored_board);

    stored_board
}
