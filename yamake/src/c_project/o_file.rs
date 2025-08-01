use crate::c_project::c_compile::object_file_from_cfile;
use crate::model as M;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Ofile {
    target: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl Ofile {
    pub fn new(
        target: PathBuf,
        include_paths: Vec<PathBuf>,
    ) -> Result<Ofile, Box<dyn std::error::Error>> {
        Ok(Ofile {
            target,
            include_paths,
        })
    }
}

impl M::GNode for Ofile {
    fn build(
        &self,
        sandbox: PathBuf,
        sources: Vec<(PathBuf, String)>,
        _deps: Vec<PathBuf>,
        stdout: PathBuf,
        stderr: PathBuf,
    ) -> bool {
        match object_file_from_cfile(
            sandbox,
            self.target(),
            sources,
            self.include_paths.clone(),
            stdout,
            stderr.clone(),
        ) {
            Ok(b) => b,
            Err(e) => {
                std::fs::write(stderr.clone(), format!("{:?}", e)).expect("write to stderr file");
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
        "o file".to_string()
    }
    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
