use crate::adapters::presenters::nil_presenter::test::NilPresenter;
use crate::adapters::storages::IssueStorage;
use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
use crate::application::board::test_utils::check_boards_are_equal;
use crate::application::usecase::usecase::{IssueStorageHolder, PresenterHolder};

fn check_stored_board_matches_presented<T: PresenterHolder<NilPresenter> + IssueStorageHolder<MemoryIssueStorage>>(use_case: &T) {
    let stored_board = use_case.storage().load();
    let presented_board = use_case.presenter().last_board_rendered.as_ref().expect("board to be presented");
    check_boards_are_equal(&presented_board, &stored_board);
}
