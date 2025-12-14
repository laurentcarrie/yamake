use crate::model as M;
use crate::rules::tex_rules::tex_scan::tex_file_scan;
use std::path::PathBuf;
use std::process::Command;

// ANCHOR: structofile

#[derive(Debug, Clone)]
pub struct Pdffile {
    target: PathBuf,
    include_paths: Vec<PathBuf>,
    flags: Vec<String>,
}

// ANCHOR_END: structofile

impl Pdffile {
    pub fn new(
        target: PathBuf,
        include_paths: Vec<PathBuf>,
        flags: Vec<String>,
    ) -> Result<Pdffile, Box<dyn std::error::Error>> {
        Ok(Pdffile {
            target,
            include_paths,
            flags,
        })
    }
}

impl M::GNode for Pdffile {
    fn build(
        &self,
        _sandbox: PathBuf,
        sources: Vec<M::PathWithTag>,
        stdout: std::fs::File,
        stderr: std::fs::File,
    ) -> bool {
        log::info!("build pdffile {:?}", self.target());
        // sources has both sources and scanned deps, so one .c file and all the .h scanned deps
        let sources = sources
            .iter()
            .filter(|x| x.tag == "tex file")
            .filter(|x| {
                x.path
                    .file_name()
                    .expect("file name")
                    .to_str()
                    .expect("file name")
                    == String::from("main.tex")
            })
            .collect::<Vec<_>>();
        if sources.len() != 1 {
            log::error!(
                "bad graph construct for node {:?}, sources.len()={}",
                self,
                sources.len()
            );
            return false;
        }
        let source = sources.get(0).expect("one node").path.clone();
        log::info!("source : {:?}", &source);

        let mut binding = Command::new("lualatex");
        let binding = binding
            .arg("--interaction=nonstopmode")
            .args(self.flags.clone())
            .arg(&source)
            .arg(self.target())
            .current_dir(
                source
                    .clone()
                    .parent()
                    .expect("parent")
                    .to_str()
                    .expect("parent"),
            )
            .stdout(stdout)
            .stderr(stderr);
        let child = binding;
        log::info!("{:?}", child);
        match child.status() {
            Ok(e) => e.success(),
            Err(_e) => {
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
        let sources = sources
            .iter()
            .filter(|x| x.tag == "tex file")
            .map(|x| x.path.clone())
            .collect::<Vec<_>>();
        let mut deps: Vec<PathBuf> = vec![];
        for source in &sources {
            let mut this_deps =
                tex_file_scan(srcdir.clone(), source.clone(), self.include_paths.clone())?;
            deps.append(&mut this_deps.0);
            for d in this_deps.1 {
                let mut found = false;
                for s in &sources {
                    if &d == s {
                        found = true;
                        break;
                    }
                }
                if !found {
                    log::info!("scanned tex dep not found : {:?}", d);
                    // return Err(format!("scanned tex dep not found : {:?}", d).into());
                }
            }
        }

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
        "pdf file".to_string()
    }
    // ANCHOR_END: tag

    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
