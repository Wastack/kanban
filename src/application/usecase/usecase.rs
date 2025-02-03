use crate::adapters::storages::IssueStorage;
use crate::application::ports::presenter::Presenter;

pub trait PresenterHolder<P: Presenter> {
    fn presenter(&self) -> &P;
}

pub trait IssueStorageHolder<S: IssueStorage> {
    fn storage(&self) -> &S;
}

