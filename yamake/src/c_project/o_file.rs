use crate::c_project::c_scan::c_file_scan;
use crate::model as M;
use std::path::PathBuf;
use std::process::Command;

// ANCHOR: structofile

#[derive(Debug, Clone)]
pub struct Ofile {
    target: PathBuf,
    include_paths: Vec<PathBuf>,
    flags: Vec<String>,
}

// ANCHOR_END: structofile

impl Ofile {
    pub fn new(
        target: PathBuf,
        include_paths: Vec<PathBuf>,
        flags: Vec<String>,
    ) -> Result<Ofile, Box<dyn std::error::Error>> {
        Ok(Ofile {
            target,
            include_paths,
            flags,
        })
    }
}

impl M::GNode for Ofile {
    fn build(
        &self,
        sandbox: PathBuf,
        sources: Vec<M::PathWithTag>,
        stdout: std::fs::File,
        stderr: std::fs::File,
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

        let mut binding = Command::new("gcc");
        let mut binding = binding
            .arg("-c")
            .args(self.flags.clone())
            .arg(source)
            .arg("-o")
            .arg(self.target())
            .current_dir(&sandbox)
            .stdout(stdout)
            .stderr(stderr);
        for pi in &self.include_paths {
            binding = binding.arg("-I").arg(pi);
        }
        let child = binding;
        log::info!("{:?}", child);
        match child.status() {
            Ok(e) => e.success(),
            Err(e) => {
                // writeln!(stderr, "{:?}", e).expect("write error");
                false
            }
        }
        // if child.status()?.success() {
        //     true
        // } else {
        //     log::error!("child is : {:?}", &child);
        //     log::error!("exit : {:?}", child.status());
        //     false
        // }
    }

    // ANCHOR: scan
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
    // ANCHOR_END: scan

    // ANCHOR: target
    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    // ANCHOR_END: target

    // ANCHOR: tag
    fn tag(&self) -> String {
        "o file".to_string()
    }
    // ANCHOR_END: tag

    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
