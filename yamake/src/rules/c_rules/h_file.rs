// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use crate::model::{self as M};
// use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Hfile {
    target: PathBuf,
}

impl Hfile {
    pub fn new(target: PathBuf) -> Result<Hfile, Box<dyn std::error::Error>> {
        Ok(Hfile { target })
    }
}

impl M::GNode for Hfile {
    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "h file".to_string()
    }
}
