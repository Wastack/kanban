use crate::adapters::storages::FileStorage;
use crate::{IssueStorage, Presenter, TabularTextRenderer};

pub mod add;
pub mod delete;

// TODO do something with this

impl Default for Box<dyn IssueStorage> {
    fn default() -> Self {
        return Box::new(FileStorage::default())
    }
}

impl Default for Box<dyn Presenter> {
    fn default() -> Self {
        return Box::new(TabularTextRenderer::default())
    }
}
