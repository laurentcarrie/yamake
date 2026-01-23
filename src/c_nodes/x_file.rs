use std::path::PathBuf;
use std::process::Command;
use crate::model::GNode;

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
    fn build(&self, sandbox: &PathBuf, predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
        let inputs: Vec<PathBuf> = predecessors.iter().map(|p| sandbox.join(p.pathbuf())).collect();

        let mut cmd = Command::new("gcc");
        cmd.arg("-o").arg(sandbox.join(&self.name));
        for input in &inputs {
            cmd.arg(input);
        }

        println!("Running: {:?}", cmd);

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
