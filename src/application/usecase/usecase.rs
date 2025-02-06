use crate::adapters::storages::IssueStorage;
use crate::application::domain::error::{DomainResult, DomainResultMultiError};
use crate::application::domain::historized_board::HistorizedBoard;
use crate::application::ports::presenter::Presenter;
use crate::application::Issue;

pub trait HasPresenter<P: Presenter> {
    fn presenter_ref(&self) -> &P;
}

pub trait HasStorage<S: IssueStorage> {
    fn storage_ref(&self) -> &S;
}

pub fn with_board_saved_and_presented<U, P: Presenter, S: IssueStorage, F>(use_case: &U, f: F)
where
    F: FnOnce(HistorizedBoard<Issue>) -> HistorizedBoard<Issue>,
    U: HasPresenter<P> + HasStorage<S>,
{
    let board = use_case.storage_ref().load();
    let board = f(board);

    use_case.storage_ref().save(&board);
    use_case.presenter_ref().render_board(&board);
}

pub fn with_board_saved_and_presented_single_error<U, P: Presenter, S: IssueStorage, F>(use_case: &U, f: F)
where
    F: FnOnce(HistorizedBoard<Issue>) -> DomainResult<HistorizedBoard<Issue>>,
    U: HasPresenter<P> + HasStorage<S>,
{
    let board = use_case.storage_ref().load();
    let result = f(board);

    // ToDo: consolidate Presenter to be one method
    match result {
        Ok(board) => {
            use_case.storage_ref().save(&board);
            use_case.presenter_ref().render_board(&board);
        },
        Err(error) => {
            use_case.presenter_ref().render_error(&error);
        }
    }

}

// ToDo: consolidate multi error
pub fn with_board_saved_and_presented_multi_error<U, P: Presenter, S: IssueStorage, F>(use_case: &U, f: F)
where
    F: FnOnce(HistorizedBoard<Issue>) -> DomainResultMultiError<HistorizedBoard<Issue>>,
    U: HasPresenter<P> + HasStorage<S>,
{
    let board = use_case.storage_ref().load();
    let result = f(board);

    // ToDo: consolidate Presenter to be one method
    match result {
        Ok(board) => {
            use_case.storage_ref().save(&board);
            use_case.presenter_ref().render_board(&board);
        },
        Err(errors) => {
            use_case.presenter_ref().render_errors(&errors);
        }
    }

}
