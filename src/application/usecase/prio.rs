use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::{PrioCommand};


#[derive(Default)]
pub(crate) struct PrioUseCase<I: IssueStorage, P: Presenter> {
    storage: I,
    presenter: P,
}

impl<I: IssueStorage, P: Presenter> PrioUseCase<I, P> {
    pub(crate) fn execute(&mut self, index: usize, command: PrioCommand) {
        let mut board = self.storage.load();

        if let Err(err) = board.get_by_index(index) {
            self.presenter.render_error(&err);
            return
        }

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
    }
}

/*

TODO: tests

- Top, Bottom, Up, Down successful cases
- Successful, but didn't change order (e.g. when calling top on an issue already there)
- index out of range
 */