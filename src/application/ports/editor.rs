use std::io;

pub(crate) trait Editor {
    /// Allows text to be edited, resulting text is returned.
    ///
    /// TODO Abstract IO::Error?
    fn open_editor_with(&self, text: &str) -> Result<String, io::Error>;
}
