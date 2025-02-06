use internal_macros::{PresenterHolder, StorageHolder};
use crate::application::{Issue, State};
use crate::application::domain::history::UndoableHistoryElement;
use crate::application::domain::parse_date::DateParser;
use crate::application::issue::Description;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::application::ports::time::{TodayProvider};
use crate::application::usecase::usecase::{HasStorage, HasPresenter, with_board_saved_and_presented_single_error};


// ToDo: use use-case traits

#[derive(Default, PresenterHolder, StorageHolder)]
pub(crate) struct AddUseCase<I: IssueStorage, P: Presenter, T: TodayProvider> {
    pub(crate) storage: I,
    presenter: P,
    time_provider: T,
}

impl<I: IssueStorage, P: Presenter, T: TodayProvider> AddUseCase<I, P, T> {
    pub(crate) fn execute(&self, description: &str, state: State, due_date: Option<String>) {
        with_board_saved_and_presented_single_error(self, |mut board| {
            let date_parser = DateParser {
                today_provider: &self.time_provider,
            };

            let due_date = due_date.map(|due_text| date_parser.parse(due_text.as_str()))
                .transpose()?;

            board.append_entity(Issue{
                description: Description::from(description),
                state,
                time_created: self.time_provider.today(),
                due_date,
            });
            board.history.add(UndoableHistoryElement::Add);

            Ok(board)
        });
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use time::macros::date;
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::adapters::time_providers::fake::{FakeTodayProvider, DEFAULT_FAKE_TODAY};
    use crate::application::{Issue, State};
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::UndoableHistoryElement;
    use crate::application::issue::Description;
    use crate::application::usecase::add::AddUseCase;
    use crate::application::usecase::test_utils::get_stored_and_presented_board;

    #[test]
    fn test_successful_add_use_case() {
        let add_use_case = given_add_use_case_with(
            HistorizedBoard::default().with_4_typical_issues(),
        );

        add_use_case.execute("New task", State::Review, Some(String::from("2023-01-02")));

        let stored_board = get_stored_and_presented_board(&add_use_case);


        stored_board.assert_issue_count(5);

        let issue = stored_board.get_with_index(0);
        check!(issue.description == Description::from("New task"), "Expected specific description of added issue");
        check!(issue.state == State::Review, "Expected specific state of added issue");
        check!(issue.time_created == DEFAULT_FAKE_TODAY, "Expected creation time to be set");
        check!(issue.due_date == Some(date!(2023-01-02)));

        let history = stored_board.history.last();
        let_assert!(Some(history) = history, "Expected to have an item in history");
        assert_eq!(history, &UndoableHistoryElement::Add, "Expected item in history to represent and addition of an issue");
    }

    // ToDo: failure for Add use case?

    fn given_add_use_case_with(board: HistorizedBoard<Issue>) -> AddUseCase<MemoryIssueStorage, NilPresenter, FakeTodayProvider> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        AddUseCase {
            storage,
            ..Default::default()
        }
    }
}