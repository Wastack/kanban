use crate::application::{elapsed_time_since_epoch, Issue, State};
use crate::application::issue::Description;
use crate::application::ports::issue_storage::IssueStorage;
use crate::adapters::storages::home_file_storage;
use crate::application::ports::presenter::Presenter;
use crate::adapters::presenters::stdoutrenderer::TabularTextRenderer;


pub(crate) struct AddUseCase {
}

impl AddUseCase {
    pub(crate) fn execute(description: String, state: State) {
            let storage = home_file_storage();
            let mut board = storage.load();

            let description = String::from(description.trim());

            board.issues.insert(0, Issue{
                description: Description(description),
                state,
                time_created: elapsed_time_since_epoch(),
            });

            storage.save(&board);
            TabularTextRenderer::default().render_board(&board);
    }

}