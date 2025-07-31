use log;
// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use crate::model as M;
use petgraph::graph::NodeIndex;
use regex::Regex;
use std::process::Command;
// use std::sync::Arc;
use crate::c_project::c_link::exe_from_obj_files;

#[derive(Debug, Clone)]
pub struct Xfile {
    target: PathBuf,
}

impl Xfile {
    pub fn new(target: PathBuf) -> Result<Xfile, Box<dyn std::error::Error>> {
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
        Ok(Xfile { target })
    }
}

impl M::GNode for Xfile {
    fn build(
        &self,
        sandbox: PathBuf,
        sources: Vec<(PathBuf, String)>,
        deps: Vec<PathBuf>,
        stdout: PathBuf,
        stderr: PathBuf,
    ) -> bool {
        match exe_from_obj_files(sandbox, self.target(), sources, stdout, stderr.clone()) {
            Ok(success) => success,
            Err(e) => {
                std::fs::write(stderr.clone(), format!("{:?}", e));
                false
            }
        }
    }

    fn scan(
        &self,
        _srcdir: PathBuf,
        _source: PathBuf,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        unimplemented!()
    }

    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "x file".to_string()
    }
}
