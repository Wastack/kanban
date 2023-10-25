
use std::process::{Command, Stdio};
use std::env;
use std::fs::File;
use std::io::{self, Error, Read, Write};
use crate::application::ports::editor::Editor;


pub(crate) struct OsDefaultEditor {}

impl Editor for OsDefaultEditor {
    fn open_editor_with(&self, text: &str) -> Result<String, Error> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vim"));

        let tempfile = tempfile::Builder::new()
            .suffix(".txt")
            .tempfile()?;

        let tempfile_path = tempfile.path();

        {
            let mut file = File::create(tempfile_path)?;
            file.write_all(text.as_bytes())?;
            // File is closed when variable goes out of scope
        }


        let tempfile_path = tempfile_path.to_str().unwrap();


        let status = Command::new(editor)
            .arg(tempfile_path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Editor returned non-zero exit status"));
        }

        let mut contents = String::new();
        let mut file = File::open(tempfile_path)?;
        file.read_to_string(&mut contents)?;

        // TODO do I allow multiline string?
        Ok(String::from(contents.replace("\n", " ").trim()))
    }
}
