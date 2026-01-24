use crate::model::GNode;
use log::info;
use std::path::PathBuf;
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
    fn build(&self, sandbox: &PathBuf, predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
        let inputs: Vec<PathBuf> = predecessors
            .iter()
            .map(|p| sandbox.join(p.pathbuf()))
            .collect();

        let mut cmd = Command::new("ar");
        cmd.arg("rcs");
        cmd.arg(sandbox.join(&self.name));
        for input in &inputs {
            cmd.arg(input);
        }

        info!("Running: {:?}", cmd);

        match cmd.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "AFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
