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

        let _id = board.find_entity_id_by_index(index)
            .inspect_err(|e| self.presenter.render_error(e))?;

        // todo: use id instead of index
        // TODO move back here storing stuff from domain?
        match command {
            PrioCommand::Top => { board.prio_top_in_category(index); },
            PrioCommand::Bottom => board.prio_bottom_in_category(index),
            PrioCommand::Up => board.prio_up_in_category(index),
            PrioCommand::Down => board.prio_down_in_category(index),
        }

        // TODO: add history

        self.storage.save(&board);
        self.presenter.render_board(&board);

        Ok(())
    }
}

/*

TODO: tests

- Top, Bottom, Up, Down successful cases
- Successful, but didn't change order (e.g. when calling top on an issue already there)
- index out of range
 */