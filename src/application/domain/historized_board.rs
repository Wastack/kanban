use std::ops::{Deref, DerefMut};
use crate::application::board::Board;
use crate::application::domain::history::{Historized, History};
use crate::application::issue::{IdGenerator, UUidGenerator};

#[derive(Debug, Clone)]
pub struct HistorizedBoard<T: Historized, IdGen: IdGenerator = UUidGenerator> {
    pub board: Board<T, IdGen>,

    pub history: History<T::HistoryType>,
}

impl<T: Historized, IdGen: IdGenerator> Deref for HistorizedBoard<T, IdGen> {
    type Target = Board<T, IdGen>;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}

impl<T: Historized, IdGen: IdGenerator> DerefMut for HistorizedBoard<T, IdGen> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.board
    }
}

impl<T: Historized, IdGen: IdGenerator> Default for HistorizedBoard<T, IdGen> {
    // Because of the generic type, derive for `Default` didn't work
    fn default() -> Self {
        Self {
            board: Board::default(),

            history: Default::default(),
        }
    }
}


impl<T: Historized, IdGen: IdGenerator> HistorizedBoard<T, IdGen> {
    pub(crate) fn new(entities: Vec<T>, deleted_entities: Vec<T>, history: Vec<T::HistoryType>) -> Self {
        Self {
            board: Board::new(entities, deleted_entities),
            history: History {
                stack: history,
            },
        }
    }
}
