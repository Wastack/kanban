use std::marker::PhantomData;
use internal_macros::{PresenterHolder, StorageHolder};
use uuid::Uuid;
use crate::application::board::Board;
use crate::application::ports::issue_storage::IssueStorage;
use crate::application::ports::presenter::Presenter;
use crate::application::domain::error::DomainResult;
use crate::application::domain::history::{PrioHistoryElement, UndoableHistoryElement};
use crate::application::Issue;
use crate::application::usecase::usecase::{HasStorage, HasPresenter};

pub(crate) trait PriorityModifier: Default {
    fn modify_priority(board: &mut Board<Issue>, id: Uuid);
}

#[derive(Default)]
pub(crate) struct TopPriority{}

impl PriorityModifier for TopPriority {
    fn modify_priority(board: &mut Board<Issue>, id: Uuid) {
        board.prio_top_in_category(id);
    }
}

#[derive(Default)]
pub(crate) struct BottomPriority{}

impl PriorityModifier for BottomPriority {
    fn modify_priority(board: &mut Board<Issue>, id: Uuid) {
        board.prio_bottom_in_category(id);
    }
}

#[derive(Default)]
pub(crate) struct UpPriority{}

impl PriorityModifier for UpPriority {
    fn modify_priority(board: &mut Board<Issue>, id: Uuid) {
        board.prio_up_in_category(id);
    }
}

#[derive(Default)]
pub(crate) struct DownPriority{}

impl PriorityModifier for DownPriority {
    fn modify_priority(board: &mut Board<Issue>, id: Uuid) {
        board.prio_down_in_category(id);
    }
}

// ToDo: use use-case traits
#[derive(Default, PresenterHolder, StorageHolder)]
pub(crate) struct PriorityUseCase<I: IssueStorage, P: Presenter, PM: PriorityModifier> {
    storage: I,
    presenter: P,

    _priority_modifier: PhantomData<PM>
}
impl<I: IssueStorage, P: Presenter, PM: PriorityModifier> PriorityUseCase<I, P, PM> {
    pub(crate) fn execute(&self, index: usize) {
        let _ = self.try_execute(index)
            .inspect_err(|e| self.presenter.render_error(e));
    }

    fn try_execute(&self, index: usize) -> DomainResult<()> {
        let mut historized_board = self.storage.load();

        let id = historized_board.find_entity_id_by_index(index)?;

        PM::modify_priority(&mut historized_board.board, id);

        let new_index = historized_board.position(id);

        if index != new_index {
            historized_board.history.add(UndoableHistoryElement::Prio(PrioHistoryElement{
                original_index: index,
                new_index,
            }));
        }

        self.storage.save(&historized_board);
        self.presenter.render_board(&historized_board);

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use assert2::let_assert;
    use crate::adapters::presenters::nil_presenter::test::NilPresenter;
    use crate::adapters::storages::IssueStorage;
    use crate::adapters::storages::memory_issue_storage::test::MemoryIssueStorage;
    use crate::adapters::time_providers::fake::DEFAULT_FAKE_TODAY;
    use crate::application::domain::historized_board::HistorizedBoard;
    use crate::application::{Issue, State};
    use crate::application::board::test_utils::check_boards_are_equal;
    use crate::application::domain::error::DomainError;
    use crate::application::domain::history::{PrioHistoryElement, UndoableHistoryElement};
    use crate::application::issue::Description;
    use crate::application::usecase::prio::{BottomPriority, DownPriority, PriorityModifier, PriorityUseCase, TopPriority, UpPriority};
    use crate::application::usecase::test_utils::{check_no_errors, get_stored_and_presented_board};
    use crate::application::usecase::usecase::HasPresenter;

    #[test]
    fn test_prio_top() {
        let use_case = given_prio_use_case_with::<TopPriority>(simple_board());

        // when
        use_case.execute(1);

        // then
        check_no_errors(&use_case);

        let stored_board = get_stored_and_presented_board(&use_case);

        let history = stored_board.history.stack.as_slice();

        let_assert!([UndoableHistoryElement::Prio( PrioHistoryElement{ original_index: 1, new_index: 0, } )] = history);

        check_issues_are_swapped(&stored_board);
    }

    #[test]
    fn test_prio_index_out_of_range() {
        let use_case = given_prio_use_case_with::<TopPriority>(simple_board());

        // when
        use_case.execute(2);

        // then
        let cell = use_case.presenter_ref().errors_presented.borrow();
        let error = cell.last();
        let_assert!(Some(DomainError::IndexOutOfRange(2)) = error);
    }

    #[test]
    fn test_prio_successful_no_order_change() {
        let use_case = given_prio_use_case_with::<TopPriority>(simple_board());

        // when
        use_case.execute(0);

        // then
        check_no_errors(&use_case);

        let cell = use_case.presenter_ref().last_board_rendered.borrow();
        let displayed_board = cell.as_ref().expect("Expected board to be displayed");

        check_boards_are_equal(&simple_board(), &displayed_board); // remained the same
        check_boards_are_equal(&displayed_board, &use_case.storage.load());
    }

    #[test]
    fn test_prio_bottom() {
        let use_case = given_prio_use_case_with::<BottomPriority>(simple_board());

        // when
        use_case.execute(0);

        // then
        check_no_errors(&use_case);

        let stored_board = get_stored_and_presented_board(&use_case);
        check_issues_are_swapped(&stored_board);
    }
    #[test]
    fn test_prio_up() {
        let use_case = given_prio_use_case_with::<UpPriority>(simple_board());

        // when
        use_case.execute(1);

        // then
        check_no_errors(&use_case);

        let stored_board = get_stored_and_presented_board(&use_case);

        check_issues_are_swapped(&stored_board);
    }

    #[test]
    fn test_prio_down() {
        let use_case = given_prio_use_case_with::<DownPriority>(simple_board());

        // when
        use_case.execute(0);

        // then
        check_no_errors(&use_case);

        let cell = use_case.presenter.last_board_rendered.borrow();
        let displayed_board = cell.as_ref().expect("Expected board to be displayed");

        check_issues_are_swapped(&displayed_board);
        check_boards_are_equal(&displayed_board, &use_case.storage.load());
    }

    fn check_issues_are_swapped(displayed_board: &HistorizedBoard<Issue>) {
        for (expected_index, expected_description) in [
            (0, "Second Issue"),
            (1, "First Issue")
        ] {
            let actual = displayed_board.get(displayed_board.find_entity_id_by_index(expected_index).expect("Expected to find issue with index"));
            assert_eq!(actual.description, expected_description.into());
        }
    }


    fn simple_board() -> HistorizedBoard<Issue> {
        HistorizedBoard::new(
            [
                ("First Issue", State::Open),
                ("Second Issue", State::Open)
            ].into_iter().map(|(d, state)| Issue { description: Description::from(d), state,
                time_created: DEFAULT_FAKE_TODAY,
                due_date: None,
            }).collect(),
            vec![],
            vec![])
    }

    fn given_prio_use_case_with<PM: PriorityModifier>(board: HistorizedBoard<Issue>) -> PriorityUseCase<MemoryIssueStorage, NilPresenter, PM> {
        let storage = MemoryIssueStorage::default();
        storage.save(&board);

        PriorityUseCase {
            storage,
            ..Default::default()
        }
    }
}
