use argh::FromArgs;
use log;
use simple_logging;
use std::process::{ExitStatus, Stdio};
// use tokio::process::Child;
use std::path::{Path, PathBuf};
// use tokio::process::Command;
use std::process::Command;
use walkdir::WalkDir;

use petgraph::dot::Dot;
use yamake::model as M;
use yamake::run::make;

pub struct CSource;

// async fn object_file_from_cfile(
//     target_file: PathBuf,
//     sources: Vec<(PathBuf, String)>,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     true
// }

pub fn object_file_from_cfile(
    sandbox: PathBuf,
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("compile C file {:?}", target_file);
    if sources.len() != 1 {
        return Err("bad length for build".into());
    };
    let source = sources.first().ok_or("empty sources")?;
    if source.1 != ".c".to_string() {
        return Err("bad build of graph".into());
    };
    let source = source.0.clone();

    log::info!("compile, source is : {:?}", source.file_name());
    log::info!("compile, target : {:?}", target_file);

    let mut binding = Command::new("gcc");
    let binding=binding
        .arg("-c")
        .arg(source.file_name().ok_or("huh, no filename ?")?)
        .arg("-o")
        .arg(target_file)
        .current_dir(sandbox)
        // .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // .spawn()?
        // .try_wait()?
        // .await?;
        ;
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
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut binding = Command::new("gcc");
    let binding =binding
        .args(sources.iter().map(|(s, _)| s).collect::<Vec<_>>())
        .arg("-o")
        .arg(target_file)
                .current_dir(sandbox)

        // .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // .spawn()?
        // .wait()
        // .await?;
        ;
    let child = binding;
    log::info!("child is : {:?}", &child);
    log::info!("exit : {:?}", child.status());

    Ok(true)
}

/// arguments for make
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "8")]
    nb_workers: u32,
    /// force rebuild
    #[argh(switch, short = 'f')]
    force: bool,
    /// the rootdir of the sources
    #[argh(positional)]
    srcdir: String,
    /// the sandbox directory where the build will take place
    #[argh(positional)]
    sandbox: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //  simple_logging::log_to_stderr(LevelFilter::Info) ;
    // simple_logger::init_with_level(log::Level::Info).unwrap();
    simple_logging::log_to_file("make.log", log::LevelFilter::Info)?;

    let cli: Cli = argh::from_env();

    let srcdir = PathBuf::from(cli.srcdir);
    let sandbox = PathBuf::from(cli.sandbox);

    let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;

    let c_file = ".c".to_string();
    let h_file = ".h".to_string();

    let o_file = ".o".to_string();
    let exe_file = "exe".to_string();

    // populate the graph with source code.
    // in this demo, we walk in the srcdir and add all the .c and .h files
    for entry in WalkDir::new(srcdir).into_iter() {
        if let Ok(entry) = entry
            && entry.path().is_file()
        {
            if let Some(s) = entry.path().extension() {
                match s.to_str() {
                    Some("c") => g.add_root_node(entry.path().to_path_buf(), c_file.clone())?,
                    Some("h") => g.add_root_node(entry.path().to_path_buf(), h_file.clone())?,
                    s => log::info!("ignored entry : {:?}", s),
                }
            }
        }
    }

    // as in Makefile where .c.o is an implicit rule,
    // for each .c file we add a .o file the explicit dependecy and the build function
    for (source, nid) in g.map.clone() {
        if source.extension().map(|x| x.to_str()).flatten() == Some("c") {
            let target = source.with_extension("o");
            g.add_node(target.clone(), o_file.clone(), object_file_from_cfile)?;
            g.add_edge(target.clone().with_extension("c"), target.clone())?;
        }
    }

    for (source, nid) in &g.map {
        log::info!("map : {} => {:?}", source.display(), nid);
    }

    // let main_c = g.add_root_node(PathBuf::from("main.c"), c_file.clone())?;
    // let add_c = g.add_root_node(PathBuf::from("add.c"), c_file.clone())?;
    // let add_h = g.add_root_node(PathBuf::from("add.h"), h_file.clone())?;

    // let main_o = g.add_node(
    //     PathBuf::from("main.o"),
    //     o_file.clone(),
    //     // M::convert_fn(object_file_from_cfile),
    //     object_file_from_cfile,
    // )?;

    // let add_o = g.add_node(
    //     PathBuf::from("add.o"),
    //     o_file.clone(),
    //     // M::convert_fn(object_file_from_cfile),
    //     object_file_from_cfile,
    // )?;

    // let exe = g.add_node(
    //     PathBuf::from("main"),
    //     exe_file.clone(),
    //     // M::convert_fn(object_file_from_cfile),
    //     exe_from_obj_files,
    // )?;

    // g.add_edge(main_o, main_c)?;
    // g.add_edge(add_o, add_c)?;
    // g.add_edge(exe, main_o)?;
    // g.add_edge(exe, add_o)?;

    let basic_dot = Dot::new(&g.g);

    let pdot = PathBuf::from("out.dot");
    std::fs::write(pdot, format!("{:?}", basic_dot))?;

    log::info!("main foo");
    match make(&mut g, cli.force, cli.nb_workers).await {
        Ok(()) => (),
        Err(e) => println!("{}", e.to_string()),
    };
    Ok(())
}
