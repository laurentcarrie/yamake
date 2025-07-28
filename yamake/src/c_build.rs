use log;
// use tokio::process::Child;
use std::{io::Write, path::PathBuf};
// use tokio::process::Command;
use crate::util::logstream;
use petgraph::graph::NodeIndex;
use std::process::Command;

use crate::model as M;

pub fn object_file_from_cfile(
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
    id: NodeIndex,
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
