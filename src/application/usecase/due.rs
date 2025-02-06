use internal_macros::{PresenterHolder, StorageHolder};
use crate::adapters::storages::IssueStorage;
use crate::application::domain::error::{DomainResult};
use crate::application::domain::history::{DueHistoryElement, UndoableHistoryElement};
use crate::application::domain::parse_date::DateParser;
use crate::application::ports::presenter::Presenter;
use crate::application::ports::time::TodayProvider;
use crate::application::usecase::usecase::{HasPresenter, HasStorage};

#[derive(Default, PresenterHolder, StorageHolder)]
pub(crate) struct DueUseCase<I: IssueStorage, P: Presenter, T: TodayProvider> {
    pub(crate) storage: I,
    presenter: P,
    today_provider: T,
}

impl<I: IssueStorage, P: Presenter, T:TodayProvider> DueUseCase<I, P, T> {
    pub(crate) fn execute(&self, index: usize, date: Option<&str>) {
        let _ = self.try_execute(index, date)
            .inspect_err(|e| self.presenter.render_error(e));
    }

    fn try_execute(&self, index: usize, date: Option<&str>) -> DomainResult<()> {
        let mut board = self.storage.load();
        let id = board.find_entity_id_by_index(index)?;

        let date_parser = DateParser {
            today_provider: &self.today_provider,
        };

        let parsed_date = date.map(|d| date_parser.parse(d)).transpose()?;

        let previous_due = board.get(id).due_date;
        board.get_mut(id).due_date = parsed_date;

        let undo_item = UndoableHistoryElement::Due(DueHistoryElement{
            index,
            previous_due,
        });

        board.history.add(undo_item);

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use assert2::{check, let_assert};
    use time::macros::date;
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::adapters::time_providers::fake::{FakeTodayProvider, DEFAULT_FAKE_TODAY};
    use crate::application::domain::error::DomainError;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::{Issue, State};
    use crate::application::domain::history::{DueHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::due::{DueUseCase};
    use crate::application::usecase::test_utils::get_stored_and_presented_board;
    use crate::application::usecase::usecase::HasPresenter;

    #[test]
    fn test_typical_due() {
        let use_case = given_due_usecase_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        use_case.execute(1, Some("2025-01-26"));

        let board = get_stored_and_presented_board(&use_case);
        let issue = board.get_with_index(1);

        check!(issue.due_date == Some(date!(2025-01-26)));
        check!(board.history.stack.last() == Some(&UndoableHistoryElement::Due(DueHistoryElement{
            index: 1,
            previous_due: None,
        })));
    }

    #[test]
    fn test_index_error() {
        let use_case = DueUseCase::<MemoryIssueStorage, NilPresenter, FakeTodayProvider>::default();

        use_case.execute(1, None);

        let errors_presented_cell = use_case.presenter_ref().errors_presented.borrow();
        let error = errors_presented_cell.first().expect("error to be presented");
        let_assert!(DomainError::IndexOutOfRange(1) = error);
    }

    #[test]
    fn test_clear_due() {
        let use_case = given_due_usecase_with(
            HistorizedBoard::default().with_issue(due_issue()),
        );
        use_case.execute(0, None);

        let stored_board = get_stored_and_presented_board(&use_case);
        let issue = stored_board.get_with_index(0);

        check!(issue.due_date == None);
        check!(stored_board.history.stack.last() == Some(&UndoableHistoryElement::Due(DueHistoryElement{
            index: 0,
            previous_due: Some(date!(1996-01-16)),
        })));
    }

    #[test]
    fn test_overwrite_due_with_tomorrow() {
        let use_case = given_due_usecase_with(
            HistorizedBoard::default().with_issue(due_issue()),
        );
        use_case.execute(0, Some("tomorrow"));

        let stored_board = get_stored_and_presented_board(&use_case);
        let issue = stored_board.get_with_index(0);

        check!(issue.due_date == Some(date!(2025-02-23)));
        check!(stored_board.history.stack.last() == Some(&UndoableHistoryElement::Due(DueHistoryElement{
            index: 0,
            previous_due: Some(date!(1996-01-16)),
        })));

    }

    fn due_issue() -> Issue {
        Issue {
            description: Description::from("due issue"),
            state: State::Open,
            due_date: Some(date!(1996-01-16)),
            time_created: DEFAULT_FAKE_TODAY,
        }
    }

    fn given_due_usecase_with(board: HistorizedBoard<Issue>) -> DueUseCase<MemoryIssueStorage, NilPresenter, FakeTodayProvider> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        DueUseCase {
            storage,
            ..Default::default()
        }
    }

}