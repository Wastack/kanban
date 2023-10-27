#[cfg(test)]
pub(crate) mod test {
    use std::error::Error;
    use crate::application::Board;
    use crate::Presenter;


    #[derive(Default)]
    pub(crate) struct NilPresenter {
        past_errors: Vec<String>
    }

    impl Presenter for NilPresenter {
        fn render_board(&self, _: &Board) {}

        fn render_error(&mut self, err: &dyn Error) {
            self.past_errors.push(err.to_string());
        }
    }
}
