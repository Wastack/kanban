use assert2::{let_assert};
use crate::adapters::presenters::nil_presenter::test::NilPresenter;
use crate::adapters::storages::IssueStorage;
use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
use crate::application::board::test_utils::check_boards_are_equal;
use crate::application::domain::historized_board::HistorizedBoard;
use crate::application::Issue;
use crate::application::usecase::usecase::{HasStorage, HasPresenter};

pub fn get_stored_and_presented_board<T: HasPresenter<NilPresenter> + HasStorage<MemoryIssueStorage>>(use_case: &T) -> HistorizedBoard<Issue> {
    let stored_board = use_case.storage_ref().load();
    let last_board_rendered_ref = use_case.presenter_ref().last_board_rendered.borrow();
    let last_board_rendered = &last_board_rendered_ref.as_ref();
    let_assert!(Some(presented_board) = last_board_rendered);

    check_boards_are_equal(presented_board, &stored_board);
    stored_board
}


pub fn check_no_errors<T: HasPresenter<NilPresenter>>(use_case: &T) {
    let presenter = use_case.presenter_ref();

    let cell =&presenter.errors_presented.borrow();
    let errors_presented  = cell.last();
    let_assert!(None = errors_presented, "Expected no errors to be presented");
}