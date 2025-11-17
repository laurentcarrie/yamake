use crate::helpers::io::write_string;
use crate::model as M;
use crate::rules::lilypond_rules::ly_scan::ly_file_scan;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, ExitCode, ExitStatus};

#[derive(Debug, Clone)]
pub struct Lyoutputfile {
    target: PathBuf,
}

impl Lyoutputfile {
    pub fn new(target: PathBuf) -> Result<Lyoutputfile, Box<dyn std::error::Error>> {
        Ok(Lyoutputfile { target })
    }
}

impl M::GNode for Lyoutputfile {
    fn build(
        &self,
        _sandbox: PathBuf,
        sources: Vec<M::PathWithTag>,
        stdout: std::fs::File,
        stderr: std::fs::File,
    ) -> bool {
        let sources = sources
            .iter()
            .filter(|x| x.tag == "ly file")
            .collect::<Vec<_>>();
        if sources.len() != 1 {
            log::error!(
                "bad graph construct for node {:?}, sources.len()={}",
                self,
                sources.len()
            );
            return false;
        }

        // source is eg solo.ly
        // we will
        // - write a wrapper solo.lytex
        //-  use lilypond-book to generate solo.output/solo.tex

        let source = sources.get(0).expect("one node").path.clone();
        log::info!("source : {:?}", &source);
        let lyfile_no_ext: String = source
            .file_name()
            .expect("file name")
            .to_str()
            .expect("to str")
            .replace(".ly", "");
        let mut outputdir = source.clone().parent().expect("parent").to_path_buf();
        outputdir.push(&lyfile_no_ext);
        outputdir.set_extension("output");

        let mut lytexfile = source.clone();
        lytexfile.set_extension("lytex");

        write_string(
            &lytexfile,
            &format!(
                "\\lilypondfile{{{}}}",
                source.file_name().unwrap().to_str().unwrap()
            ),
        )
        .unwrap();
        let mut binding = Command::new("lilypond-book");
        let binding = binding
            .arg("--output")
            .arg(outputdir.to_str().unwrap())
            .arg("--pdf")
            .arg("--latex-program=lualatex")
            .arg(lytexfile.to_str().unwrap())
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
        log::info!("{:?}", child.status());
        match child.status() {
            Ok(e) => {
                if e.success() {
                    true
                } else {
                    log::error!("{:?}", e.code());
                    false
                }
            }
            Err(e) => {
                // writeln!(stderr, "{:?}", e).expect("write error");
                log::error!("{:?}", e);
                log::error!("{:?}", child);
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
            .filter(|x| x.tag == "ly file")
            .collect::<Vec<_>>();
        if sources.len() != 1 {
            log::error!(
                "bad graph construct for node {:?}, sources.len()={}",
                self,
                sources.len()
            );
            return Err(format!("bad graph construct for node {:?}", self).into());
        }
        let source = sources.get(0).expect("one node").path.clone();
        let deps = ly_file_scan(srcdir, source.clone(), vec![])?;
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
        "lytex file".to_string()
    }
    // ANCHOR_END: tag

    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
