use crate::model as M;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Cfile {
    target: PathBuf,
}

impl Cfile {
    pub fn new(target: PathBuf) -> Result<Cfile, Box<dyn std::error::Error>> {
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
        Ok(Cfile { target })
    }
}

impl M::GNode for Cfile {
    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "c file".to_string()
    }
    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
