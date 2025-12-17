// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use crate::model::{self as M};
// use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct N {
    target: PathBuf,
}

pub fn new(target: PathBuf) -> Result<N, Box<dyn std::error::Error>> {
    Ok(N { target })
}

impl M::GNode for N {
    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "h file".to_string()
    }
}
