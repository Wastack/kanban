use crate::cli::{Command};
use crate::{Description, elapsed_time_since_epoch, home_file_storage, Issue, Renderer, State, Storage, TabularTextRenderer};


/// A UseCase can be called to make an action on the board (e.g. adding an item).
///
/// UseCases are stateful and irreversible. Executing `call(); undo();`
/// leaves the board in the same state
///
/// TODO: except if there is a threshold for maximum number of history items
pub(crate) trait UseCase {
    /// Returns `true`, if the `UseCase` accepts the `Command`.
    fn call(_: Command) -> Option<Box<dyn UseCase>> where Self: Sized;

    /// Using the internal state of the struct, this call reverses the action of `call`
    fn undo(&self);
}

pub(crate) struct AddUseCase {
}

impl UseCase for AddUseCase {
    fn call(command: Command) -> Option<Box<dyn UseCase>> {
        if let Command::Add{description, state} = command {
            let storage = home_file_storage();
            let mut board = storage.load();

            let description = String::from(description.trim());

            board.issues.insert(0, Issue{
                description: Description(description),
                state: match state {
                    None => State::Open,
                    Some(s) => s,
                },
                time_created: elapsed_time_since_epoch(),
            });

            storage.save(&board);
            println!("{}", TabularTextRenderer::default().render_board(&board));

            Some(Box::new(AddUseCase{}))
        } else {
            None
        }
    }

    /// Undoing `AddUseCase` always means that we delete the newest issue in board (with order `0`)
    fn undo(&self) {
        let storage = home_file_storage();
        let mut board = storage.load();

        board.issues.remove(0);

        storage.save(&board);
        println!("{}", TabularTextRenderer::default().render_board(&board));
    }
}