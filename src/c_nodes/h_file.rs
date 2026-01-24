use crate::model::GRootNode;
use std::path::PathBuf;

pub struct HFile {
    pub name: String,
}

impl HFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GRootNode for HFile {
    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "HFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
