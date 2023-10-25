use crate::adapters::storages::FileStorage;
use crate::{Editor, IssueStorage, OsDefaultEditor, Presenter, TabularTextRenderer};

pub mod add;
pub mod delete;
pub mod r#move;
pub mod edit;

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

impl Default for Box<dyn Editor> {
    fn default() -> Self {
        return Box::new(OsDefaultEditor::default())
    }
}
