use crate::c_project::c_compile::object_file_from_cfile;
use crate::c_project::c_scan::c_file_scan;
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
        sources: Vec<M::PathWithTag>,
        stdout: PathBuf,
        stderr: PathBuf,
    ) -> bool {
        // sources has both sources and scanned deps, so one .c file and all the .h scanned deps
        let sources = sources
            .iter()
            .filter(|x| x.tag == "c file")
            .collect::<Vec<_>>();
        if sources.len() != 1 {
            log::error!("bad graph construct for node {:?}", self);
            return false;
        }
        let source = sources.get(0).expect("one node").path.clone();
        match object_file_from_cfile(
            sandbox,
            self.target(),
            source,
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
        srcdir: PathBuf,
        sources: Vec<M::PathWithTag>,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        // sanity check, the rule is .o : .c
        // if sources.len() != 1 {
        //     for s in sources {
        //         log::info!("source : {:?}", s);
        //     }
        //     log::error!("o_node should only have one source");

        //     return Err("o_node should only have one source".into());
        // }
        let sources = sources
            .iter()
            .filter(|x| x.tag == "c file")
            .collect::<Vec<_>>();
        if sources.len() != 1 {
            log::error!("bad graph construct for node {:?}", self);
            return Err(format!("bad graph construct for node {:?}", self).into());
        }
        let source = sources.get(0).expect("one node").path.clone();
        let deps = c_file_scan(srcdir, source.clone(), self.include_paths.clone())?;
        Ok(deps)
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
