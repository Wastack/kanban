#[cfg(test)]

use std::error::Error;
use crate::application::Board;
use crate::Presenter;


pub(crate) struct NilPresenter { }

impl Default for NilPresenter {
    fn default() -> Self { Self{} }
}

impl Presenter for NilPresenter {
    fn render_board(&self, _: &Board) {}

    fn render_error(&self, _: &dyn Error) {}
}