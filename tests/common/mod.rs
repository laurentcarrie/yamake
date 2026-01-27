mod json_desc;
mod yml_desc;

#[allow(unused_imports)]
pub use json_desc::JsonDesc;
#[allow(unused_imports)]
pub use yml_desc::YmlDesc;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use yamake::model::{GNode, GRootNode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Language {
    pub language: String,
    pub helloworld: String,
}

/// Simple source file node for testing
pub struct SourceFile {
    pub name: String,
}

impl GRootNode for SourceFile {
    fn tag(&self) -> String {
        "SourceFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}

/// Simple target file node for testing
pub struct TargetFile {
    pub path: PathBuf,
}

impl GNode for TargetFile {
    fn tag(&self) -> String {
        "TargetFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        self.path.clone()
    }

    fn build(&self, _sandbox: &std::path::Path, _predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        true
    }
}
