#[cfg(test)]
pub(crate) mod test {
    use std::cell::{RefCell};
    use crate::application::Issue;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::ports::presenter::Presenter;


    pub(crate) struct NilPresenter {
        pub(crate) errors_presented: RefCell<Vec<DomainError>>,
        pub(crate) last_board_rendered: RefCell<Option<HistorizedBoard<Issue>>>,
    }

    impl Default for NilPresenter {
        fn default() -> Self {
            Self {
                errors_presented: RefCell::new(Vec::default()),
                last_board_rendered: RefCell::new(None),
            }
        }
    }

    impl Presenter for NilPresenter {
        fn render_board(&self, board: &HistorizedBoard<Issue>) {
            self.last_board_rendered.swap(&RefCell::new(Some(board.clone())));
        }

        fn render_error(&self, err: &DomainError) {
            self.errors_presented.borrow_mut().push(err.clone_for_testing());
        }
    }

}
