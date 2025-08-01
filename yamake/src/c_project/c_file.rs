use crate::c_project::c_scan::c_file_scan;
use crate::model as M;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Cfile {
    target: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl Cfile {
    pub fn new(
        target: PathBuf,
        include_paths: Vec<PathBuf>,
    ) -> Result<Cfile, Box<dyn std::error::Error>> {
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
        Ok(Cfile {
            target,
            include_paths,
        })
    }
}

impl M::GNode for Cfile {
    fn scan(
        &self,
        srcdir: PathBuf,
        source: PathBuf,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let ret = c_file_scan(srcdir, source, self.include_paths.clone())?;
        Ok(ret)
    }

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
