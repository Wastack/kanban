use crate::adapters::storages::IssueStorage;
use crate::application::ports::presenter::Presenter;

pub trait HasPresenter<P: Presenter> {
    fn presenter_ref(&self) -> &P;
    fn presenter_mut(&mut self) -> &mut P;
}

pub trait HasStorage<S: IssueStorage> {
    fn storage_ref(&self) -> &S;
    fn storage_mut(&mut self) -> &mut S;
}

