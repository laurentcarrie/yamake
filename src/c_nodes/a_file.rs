use std::path::PathBuf;
use crate::model::GNode;

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
    fn build(&self, _sandbox: &PathBuf, predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
        let inputs: Vec<String> = predecessors.iter().map(|p| p.id()).collect();
        println!("Archiving {} from {:?}", self.name, inputs);
        true
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
