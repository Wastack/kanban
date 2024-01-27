use crate::application::{Issue, State};
use crate::application::domain::history::UndoableHistoryElement;
use crate::application::issue::Description;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct AddUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> AddUseCase<I, P> {
    pub(crate) fn execute(&mut self, description: &str, state: State) {
        let mut board = self.storage.load();

        board.add_issue(Issue::new(
            Description::from(description),
            state,
        ));
        board.history_mut().push(UndoableHistoryElement::Add);

        self.storage.save(&board);
        self.presenter.render_board(&board)
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::{AddUseCase, IssueStorage, State};
    use crate::application::Board;
    use crate::application::domain::history::UndoableHistoryElement;
    use crate::application::issue::Description;

    #[test]
    fn test_successful_add_use_case() {
        let mut add_use_case = given_add_use_case_with(
            Board::default().with_4_typical_issues(),
        );

        add_use_case.execute("New task", State::Review);

        let stored_board = add_use_case.storage.load();

        stored_board
            .assert_issue_count(5)
            .assert_first_issue_content()
            .assert_history_consists_of_one_addition();

        // TODO assert presented board
        //let presented_board = add_use_case.presenter.last_board_rendered.expect("Expected a board to be presented");
        //assert_eq!(presented_board, &stored_board, "Expected stored and presented board to be equal");
    }

    fn given_add_use_case_with(board: Board) -> AddUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        AddUseCase {
            storage,
            ..Default::default()
        }
    }

    impl Board {
        fn assert_first_issue_content(&self) -> &Self {
            let issue = self.get_issue(0).unwrap();
            assert_eq!(issue.description, Description::from("New task"), "Expected specific description of added issue");
            assert_eq!(issue.state, State::Review, "Expected specific state of added issue");

            self
        }

        fn assert_history_consists_of_one_addition(&self) -> &Self {
            let history = self.history();
            assert_eq!(history.len(), 1, "Expected to have an item in history");
            assert_eq!(history.peek().unwrap(), &UndoableHistoryElement::Add, "Expected item in history to represent and addition of an issue");

            self
        }
    }
}