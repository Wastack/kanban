use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::{PrioCommand};
use crate::application::domain::error::DomainResult;


#[derive(Default)]
pub(crate) struct PrioUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> PrioUseCase<I, P> {
    pub(crate) fn execute(&mut self, index: usize, command: PrioCommand) {
        let _ = self.try_execute(index, command)
            .inspect_err(|e| self.presenter.render_error(e));
    }

    fn try_execute(&mut self, index: usize, command: PrioCommand) -> DomainResult<()> {
        let mut board = self.storage.load();

        let id = board.find_entity_id_by_index(index)?;

        match command {
            PrioCommand::Top => { board.prio_top_in_category(id); },
            PrioCommand::Bottom => board.prio_bottom_in_category(id),
            PrioCommand::Up => board.prio_up_in_category(id),
            PrioCommand::Down => board.prio_down_in_category(id),
        }

        // TODO: add history

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use crate::adapters::controllers::PrioCommand;
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::{Issue, State};
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::issue::Description;
    use crate::application::usecase::prio::PrioUseCase;

    #[test]
    fn test_prio_top() {
        let mut use_case = given_prio_use_case_with(simple_board());

        // when
        use_case.execute(1, PrioCommand::Top);

        // then
        assert!(use_case.presenter.errors_presented.is_empty(), "Expected no errors");

        let displayed_board = use_case.presenter.last_board_rendered.expect("Expected board to be displayed");
        for (expected_index, expected_description) in [
            (0, "Second Issue"),
            (1, "First Issue")
        ] {
            let actual = displayed_board.get(displayed_board.find_entity_id_by_index(expected_index).expect("Expected to find issue with index"));
            assert_eq!(actual.description, expected_description.into());
        }

        check_boards_are_equal(&displayed_board, &use_case.storage.load());
    }

    #[test]
    fn test_prio_index_out_of_range() {
        todo!()
    }

    #[test]
    fn test_prio_successful_no_order_change() {
        todo!()
    }

    fn simple_board() -> HistorizedBoard<Issue> {
        HistorizedBoard::new(
            [
                ("First Issue", State::Open),
                ("Second Issue", State::Open)
            ].into_iter().map(|(d, state)| Issue { description: Description::from(d), state, time_created: 0, }).collect(),
            vec![],
            vec![])
    }

    fn given_prio_use_case_with(board: HistorizedBoard<Issue>) -> PrioUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        PrioUseCase {
            storage,
            ..Default::default()
        }
    }
}
