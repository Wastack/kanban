use internal_macros::{PresenterHolder, StorageHolder};
use crate::adapters::storages::IssueStorage;
use crate::application::domain::history::{FlushHistoryElement, UndoableHistoryElement};
use crate::application::ports::presenter::Presenter;
use crate::application::usecase::usecase::{HasStorage, HasPresenter};

// ToDo: use use-case traits
#[derive(Default, PresenterHolder, StorageHolder)]
pub(crate) struct FlushUseCase<I: IssueStorage, P: Presenter> {
    pub(crate) storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> FlushUseCase<I, P> {
    pub(crate) fn execute(&mut self) {
        let mut board = self.storage.load();

        let number_of_issues_affected = board.flush();

        board.history.add(UndoableHistoryElement::Flush(FlushHistoryElement{
            number_of_issues_affected,
        }));

        self.storage.save(&board);
        self.presenter.render_board(&board)
    }
}

#[cfg(test)]
mod tests {
    use assert2::{check, let_assert};
    use crate::application::Issue;
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::{FlushHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::flush::FlushUseCase;
    use crate::application::usecase::test_utils::get_stored_and_presented_board;

    #[test]
    fn test_execute_successful_flush() {
        // Given
        let mut sut = given_flush_use_case_with(
            HistorizedBoard::default().with_4_typical_issues()
        );

        // When
        sut.execute();

        // Then
        let stored_board = get_stored_and_presented_board(&sut);

        // One remains, because the done issue is not flushed
        check!(stored_board.board.entity_count() == 1);
        let_assert!(Some(entity) = stored_board.board.entities().first(), "Expected to have an item in entities");
        check!(entity.description == Description::from("Task inserted third"));

        let deleted_entities  = stored_board.get_deleted_entities();
        check!(deleted_entities.len() == 3);
        check!(deleted_entities[0].description == Description::from("Task inserted first"));
        check!(deleted_entities[1].description == Description::from("Task inserted second"));
        check!(deleted_entities[2].description == Description::from("Task inserted fourth"));

        let_assert!(Some(UndoableHistoryElement::Flush(FlushHistoryElement{ number_of_issues_affected: 3 })) = stored_board.history.last());
    }

    fn given_flush_use_case_with(board: HistorizedBoard<Issue>) -> FlushUseCase<MemoryIssueStorage, NilPresenter> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        FlushUseCase {
            storage,
            ..Default::default()
        }
    }

}