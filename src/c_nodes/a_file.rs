use crate::command::run_command;
use crate::model::GNode;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct AFile {
    pub name: String,
}

impl AFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GNode for AFile {
    fn build(&self, sandbox: &Path, predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        // Only include OFile predecessors in the archive
        let inputs: Vec<PathBuf> = predecessors
            .iter()
            .filter(|p| p.tag() == "OFile")
            .map(|p| sandbox.join(p.pathbuf()))
            .collect();

        let mut cmd = Command::new("ar");
        cmd.arg("rcs");
        cmd.arg(sandbox.join(&self.name));
        for input in &inputs {
            cmd.arg(input);
        }

        run_command(&mut cmd, sandbox, &self.name)
    }

    fn tag(&self) -> String {
        "AFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
