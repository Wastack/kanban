use crate::adapters::storages::IssueStorage;
use crate::application::domain::error::{DomainResult};
use crate::application::domain::history::{DueHistoryElement, UndoableHistoryElement};
use crate::application::domain::parse_date::DateParser;
use crate::application::ports::presenter::Presenter;
use crate::application::ports::time::TodayProvider;

#[derive(Default)]
pub(crate) struct DueUseCase<I: IssueStorage, P: Presenter, T: TodayProvider> {
    pub(crate) storage: I,
    presenter: P,
    today_provider: T,
}

impl<I: IssueStorage, P: Presenter, T:TodayProvider> DueUseCase<I, P, T> {
    pub(crate) fn execute(&mut self, index: usize, date: Option<&str>) {
        let _ = self.try_execute(index, date)
            .inspect_err(|e| self.presenter.render_error(e));
    }

    fn try_execute(&mut self, index: usize, date: Option<&str>) -> DomainResult<()> {
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
    use crate::adapters::time_providers::fake::FakeTodayProvider;
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::{Issue, State};
    use crate::application::domain::history::{DueHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::due::DueUseCase;

    #[test]
    fn test_typical_due() {
        let mut use_case = given_due_usecase_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        use_case.execute(1, Some("2025-01-26"));

        let stored_board = use_case.storage.load();
        let presented_board = use_case.presenter.last_board_rendered.expect("board to be presented");
        check_boards_are_equal(&presented_board, &stored_board);

        let stored_due_issue = stored_board.get(stored_board.find_entity_id_by_index(1).unwrap());
        check!(stored_due_issue.due_date == Some(date!(2025-01-26)));

        check!(stored_board.history.stack.last() == Some(&UndoableHistoryElement::Due(DueHistoryElement{
            index: 1,
            previous_due: None,
        })));
    }

    #[test]
    fn test_index_error() {
        let mut use_case = DueUseCase::<MemoryIssueStorage, NilPresenter, FakeTodayProvider>::default();
        use_case.execute(1, None);
        let error = use_case.presenter.errors_presented.first().expect("error to be presented");
        let_assert!(DomainError::IndexOutOfRange(1) = error);
    }

    #[test]
    fn test_clear_due() {
        let mut use_case = given_due_usecase_with(
            HistorizedBoard::default().with_issue(Issue {
                description: Description::from("due issue"),
                state: State::Open,
                due_date: Some(date!(1996-01-16)),
                ..Default::default()
            }),
        );
        use_case.execute(0, None);

        let stored_board = use_case.storage.load();
        let presented_board = use_case.presenter.last_board_rendered.expect("board to be presented");
        check_boards_are_equal(&presented_board, &stored_board);

        let issue = stored_board.get(stored_board.find_entity_id_by_index(0).unwrap());
        check!(issue.due_date == None);

        check!(stored_board.history.stack.last() == Some(&UndoableHistoryElement::Due(DueHistoryElement{
            index: 0,
            previous_due: Some(date!(1996-01-16)),
        })));
    }

    fn given_due_usecase_with(board: HistorizedBoard<Issue>) -> DueUseCase<MemoryIssueStorage, NilPresenter, FakeTodayProvider> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        DueUseCase {
            storage,
            ..Default::default()
        }
    }

    // ToDo add usecase test with fancy date input, e.g. tomorrow
}