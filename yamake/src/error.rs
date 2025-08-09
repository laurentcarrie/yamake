use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CouldNotMountFileError {
    pub target: PathBuf,
}

impl fmt::Display for CouldNotMountFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "could not mount file '{:?}' because it does not exist. Either it is misspelled, missing, or is a built artefact and you forgot to add an edge",
            self.target
        )
    }
}

impl CouldNotMountFileError {
    pub fn new(target: PathBuf) -> CouldNotMountFileError {
        CouldNotMountFileError { target }
    }
}

impl Error for CouldNotMountFileError {}
