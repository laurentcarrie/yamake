use std::path::PathBuf;
use yamake::model as M;

#[derive(Debug, Clone)]
pub struct N {
    target: PathBuf,
}

// ANCHOR_END: structofile

impl N {
    pub fn new(target: PathBuf) -> Result<N, Box<dyn std::error::Error>> {
        Ok(N { target })
    }
}

impl M::GNode for N {
    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }

    fn tag(&self) -> String {
        "yml file".to_string()
    }

    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
