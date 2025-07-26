use log;
use simple_logger;
use std::fs::File;
use std::io::prelude::*;
use std::process::{ExitStatus, Stdio};
// use tokio::process::Child;
use std::path::PathBuf;
use tokio::process::Command;

use yamake::model as M;

pub struct CSource;

pub async fn object_file_from_cfile(
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if sources.len() != 1 {
        return Err("bad length for build".into());
    };
    let source = sources.first().ok_or("empty sources")?;
    if source.1 != ".o".to_string() {
        return Err("bad build of graph".into());
    };
    let source = source.0.clone();

    let child = Command::new("gcc")
        .arg("-c")
        .arg(source.file_name().ok_or("huh, no filename ?")?)
        .arg("-o")
        .arg(target_file)
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()
        .await?;

    Ok(true)
}

pub async fn exe_from_obj_files(
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut child = Command::new("gcc")
        .args(sources.iter().map(|(s, _)| s).collect::<Vec<_>>())
        .arg("-o")
        .arg(target_file)
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()
        .await?;

    Ok(true)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //  simple_logging::log_to_stderr(LevelFilter::Info) ;
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let obj_node_type = M::NodeType {
        build: object_file_from_cfile,
    };

    let mut g = M::G::new();

    let main_c = g.add_source_node(PathBuf::from("main.c"))?;
    let add_c = g.add_source_node(PathBuf::from("add.c"))?;

    //     g.add_node(main_c,vec![]
    //         PathBuf::from("main.o"),
    //         vec!["main.c".into()],
    //         pub fn object_file_from_cfile(
    // ,
    //     )?;

    //     g.add_node(
    //         PathBuf::from("add.o"),
    //         vec!["add.c".into()],
    //         pub fn object_file_from_cfile(
    // ,
    //     )?;

    log::info!("main foo");
    Ok(())
}
