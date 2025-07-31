use log;
// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use crate::model as M;
use petgraph::graph::NodeIndex;
use regex::Regex;
use std::process::Command;
// use std::sync::Arc;
use crate::c_project::c_compile::object_file_from_cfile;

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
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
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
        deps: Vec<PathBuf>,
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
        "o file".to_string()
    }
}

// impl std::fmt::Debug for N {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Node")
//             .field("target", &self.target)
//             .field("tag", &self.tag)
//             .finish()
//     }
// }

pub fn object_file_from_Ofile(
    sandbox: PathBuf,
    id: NodeIndex,
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
    stdout: PathBuf,
    stderr: PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("compile C file {:?}; id is {}", target_file, id.index());
    if sources.len() != 1 {
        return Err("bad length of sources, should be 1".into());
    };
    let source = sources.first().ok_or("empty sources")?;
    if source.1 != "c file".to_string() {
        return Err("source has bad tag".into());
    };
    let source = source.0.clone();

    log::info!("compile, source is : {:?}", source.file_name());
    log::info!("compile, target : {:?}", target_file);

    let mut binding = Command::new("gcc");
    let binding = binding
        .arg("-c")
        .arg(source)
        .arg("-o")
        .arg(target_file)
        .current_dir(&sandbox)
        .stdout(std::fs::File::create(stdout)?)
        .stderr(std::fs::File::create(stderr)?);
    let child = binding;
    if child.status()?.success() {
        Ok(true)
    } else {
        log::error!("child is : {:?}", &child);
        log::error!("exit : {:?}", child.status());
        Ok(false)
    }
}

pub fn exe_from_obj_files(
    sandbox: PathBuf,
    _id: NodeIndex,
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
    stdout: PathBuf,
    stderr: PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut binding = Command::new("gcc");
    let binding = binding
        .args(sources.iter().map(|(s, _)| s).collect::<Vec<_>>())
        .arg("-o")
        .arg(target_file)
        .current_dir(&sandbox)
        .current_dir(&sandbox)
        .stdout(std::fs::File::create(stdout)?)
        .stderr(std::fs::File::create(stderr)?);
    let child = binding;
    log::info!("child is : {:?}", &child);
    log::info!("exit : {:?}", child.status());

    Ok(true)
}

pub fn c_file_scan(
    srcdir: PathBuf,
    target: PathBuf,
    _stdout: PathBuf,
    _stderr: PathBuf,
    // include_path: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    log::info!("scan {:?}", target);
    let mut src_target = srcdir.clone();
    src_target.push(target);
    if !src_target.exists() {
        return Err(format!("cannot scan non existing file : {:?}", src_target).into());
    }
    let data = std::fs::read_to_string(src_target)?;
    // let re = Regex::new(r###" *\#include *"(?<name>\w+)" *"###)?;
    let re = Regex::new(r###"#include *"(?<f>.*)".*"###)?;

    let mut ret: Vec<PathBuf> = vec![];

    for caps in re.captures_iter(data.as_str()) {
        log::info!("{:?}", caps);
        log::info!("scan ==> {:?}", caps.name("f"));
        let mut scanned = srcdir.clone();
        let relpath = caps.name("f").ok_or("huh ? in scan")?.as_str();
        ret.push(PathBuf::from(relpath));
        scanned.push(relpath);
        log::info!(" rel scan : {:?}", scanned);
    }

    Ok(ret)
}
