use crate::adapters::storages::IssueStorage;
use crate::application::domain::history::{FlushHistoryElement, UndoableHistoryElement};
use crate::application::ports::presenter::Presenter;

#[derive(Default)]
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
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::domain::history::{FlushHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::flush::FlushUseCase;

    #[test]
    fn test_execute_successful_flush() {
        // Given
        let mut sut = given_flush_use_case_with(
            HistorizedBoard::default().with_4_typical_issues()
        );

        // When
        sut.execute();

        // Then
        let stored_board = sut.storage.load();

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

        let presented_board = sut.presenter.last_board_rendered.expect("Expected a board to be presented");
        check_boards_are_equal(&presented_board, &stored_board);
    }

    fn given_flush_use_case_with(board: HistorizedBoard<Issue>) -> FlushUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        FlushUseCase {
            storage,
            ..Default::default()
        }
    }

}