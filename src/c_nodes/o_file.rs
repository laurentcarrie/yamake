use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use crate::model::GNode;

pub struct OFile {
    pub name: String,
}

impl OFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GNode for OFile {
    fn build(&self, sandbox: &PathBuf, predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
        let inputs: Vec<PathBuf> = predecessors.iter()
            .filter(|p| p.tag() != "HFile")
            .map(|p| sandbox.join(p.pathbuf()))
            .collect();

        println!("build {}",self.name) ;
        thread::sleep(Duration::from_secs(15));

        let mut cmd = Command::new("gcc");
        cmd.arg("-c");
        cmd.arg("-I").arg(sandbox);
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
        "OFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
