use std::path::PathBuf;
use yamake::model::GRootNode;

pub struct YmlDesc {
    pub name: String,
}

impl YmlDesc {
    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GRootNode for YmlDesc {
    fn tag(&self) -> String {
        "YmlDesc".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
