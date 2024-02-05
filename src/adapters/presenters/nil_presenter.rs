#[cfg(test)]
pub(crate) mod test {
    use crate::application::{Board, Issue};
    use crate::application::domain::error::DomainError;
    use crate::Presenter;


    #[derive(Default)]
    pub(crate) struct NilPresenter {
        pub(crate) errors_presented: Vec<DomainError>,
        pub(crate) last_board_rendered: Option<Board<Issue>>,
    }

    impl Presenter for NilPresenter {
        fn render_board(&mut self, board: &Board<Issue>) {
            self.last_board_rendered = Some(board.clone());
        }

        fn render_error(&mut self, err: &DomainError) {
            self.errors_presented.push(err.clone_for_testing());
        }
    }

}
