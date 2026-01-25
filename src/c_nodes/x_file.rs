use crate::model::GNode;
use log::info;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct XFile {
    pub name: String,
}

impl XFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GNode for XFile {
    fn build(&self, sandbox: &Path, predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        // Separate object files and libraries - libraries must come last for Linux linker
        let mut objects: Vec<PathBuf> = Vec::new();
        let mut libraries: Vec<PathBuf> = Vec::new();

        for p in predecessors {
            let path = sandbox.join(p.pathbuf());
            if p.tag() == "AFile" {
                libraries.push(path);
            } else {
                objects.push(path);
            }
        }

        let mut cmd = Command::new("gcc");
        cmd.arg("-o").arg(sandbox.join(&self.name));

        // Add object files first, then libraries
        for obj in &objects {
            cmd.arg(obj);
        }
        for lib in &libraries {
            cmd.arg(lib);
        }

        info!("Running: {cmd:?}");

        match cmd.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "XFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
