use std::io;

pub(crate) trait Editor {
    /// An editor opens to edit original `text`, resulting text is returned.
    fn open_editor_with(&self, text: &str) -> Result<String, io::Error>;
}
