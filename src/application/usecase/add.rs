use crate::application::{Issue, State};
use crate::application::domain::history::UndoableHistoryElement;
use crate::application::issue::Description;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;


#[derive(Default)]
pub(crate) struct AddUseCase {
    storage: Box<dyn IssueStorage>,
    presenter: Box<dyn Presenter>
}

impl AddUseCase {
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

        then_extended_board(&add_use_case)
            .assert_issue_count(5)
            .assert_first_issue_content()
            .assert_history_consists_of_one_addition();
    }

    fn given_add_use_case_with(board: Board) -> AddUseCase {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        AddUseCase {
            storage: Box::new(storage),
            presenter: Box::new(NilPresenter::default()),
        }
    }

    fn then_extended_board(sut: &AddUseCase) -> Board {
        sut.storage.load()
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