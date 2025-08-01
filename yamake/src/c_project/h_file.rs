// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use crate::c_project::c_scan::c_file_scan;
use crate::model as M;
// use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Hfile {
    target: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl Hfile {
    pub fn new(
        target: PathBuf,
        include_paths: Vec<PathBuf>,
    ) -> Result<Hfile, Box<dyn std::error::Error>> {
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
        Ok(Hfile {
            target,
            include_paths,
        })
    }
}

impl M::GNode for Hfile {
    fn build(
        &self,
        _sandbox: PathBuf,
        _sources: Vec<(PathBuf, String)>,
        _deps: Vec<PathBuf>,
        _stdout: PathBuf,
        _stderr: PathBuf,
    ) -> bool {
        unimplemented!()
    }

    fn scan(
        &self,
        srcdir: PathBuf,
        source: PathBuf,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        c_file_scan(srcdir, source, self.include_paths.clone())
    }

    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "h file".to_string()
    }
    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
