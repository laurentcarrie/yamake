use crate::model::GRootNode;
use std::path::PathBuf;

pub struct CFile {
    pub name: String,
}

impl CFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GRootNode for CFile {
    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "CFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
