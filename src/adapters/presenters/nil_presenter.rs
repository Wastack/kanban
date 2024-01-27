#[cfg(test)]
pub(crate) mod test {
    use crate::application::Board;
    use crate::application::domain::error::DomainError;
    use crate::Presenter;


    #[derive(Default)]
    pub(crate) struct NilPresenter {
        //pub(crate) last_error_rendered: Option<&'a DomainError>,
        //pub(crate) last_board_rendered: Option<&'a Board>,
    }

    impl Presenter for NilPresenter {
        fn render_board(&self, _board: &Board) { }

        fn render_error(&mut self, _err: &DomainError) { }
    }

}
