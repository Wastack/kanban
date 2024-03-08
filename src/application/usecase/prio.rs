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
    pub(crate) fn execute(&mut self, index: usize, command: PrioCommand) -> DomainResult<()> {
        let mut board = self.storage.load();

        let id = board.find_entity_id_by_index(index)
            .inspect_err(|e| self.presenter.render_error(e))?;

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
    use crate::application::issue::Description;
    use crate::application::usecase::prio::PrioUseCase;

    #[test]
    fn test_typical_prio() {
        let mut use_case = given_prio_use_case_with(
            // todo: clean up a bit?
            HistorizedBoard::new(
                [
                    ("First Issue", State::Open),
                    ("Second Issue", State::Open)
                ].into_iter().map(|(d, state)| Issue { description: Description::from(d), state, time_created: 0, }).collect(),
                vec![],
                vec![]));


        // when
        use_case.execute(1, PrioCommand::Top).unwrap();

        // then
        // todo: assert:
        // - moving happened
        // - board stored and presented
    }

    // todo: test index out of range

    fn given_prio_use_case_with(board: HistorizedBoard<Issue>) -> PrioUseCase<MemoryIssueStorage, NilPresenter> {
        let mut storage = MemoryIssueStorage::default();
        storage.save(&board);

        PrioUseCase {
            storage,
            ..Default::default()
        }
    }
}

/*

TODO: tests

- Top, Bottom, Up, Down successful cases
- Successful, but didn't change order (e.g. when calling top on an issue already there)
- index out of range
 */