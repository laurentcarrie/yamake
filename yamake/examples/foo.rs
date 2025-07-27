use argh::FromArgs;
use log;
use simple_logging;
use std::process::{ExitStatus, Stdio};
// use tokio::process::Child;
use std::path::PathBuf;
use tokio::process::Command;

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

    let child = Command::new("gcc")
        .arg("-c")
        .arg(source.file_name().ok_or("huh, no filename ?")?)
        .arg("-o")
        .arg(target_file)
        .current_dir(sandbox)
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .try_wait()
        // .await?;
        ;

    log::info!("SLEEP");
    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(true)
}

// pub async fn exe_from_obj_files(
//     target_file: PathBuf,
//     sources: Vec<(PathBuf, String)>,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     let mut child = Command::new("gcc")
//         .args(sources.iter().map(|(s, _)| s).collect::<Vec<_>>())
//         .arg("-o")
//         .arg(target_file)
//         .kill_on_drop(true)
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn()?
//         .wait()
//         .await?;

//     Ok(true)
// }

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

    // let obj_node_type = M::NodeType {
    //     build: object_file_from_cfile,
    // };

    let mut g = M::G::new(PathBuf::from(&cli.srcdir), PathBuf::from(&cli.sandbox));

    let c_file = ".c".to_string();
    let o_file = ".o".to_string();
    let exe_file = "exe".to_string();

    let main_c = g.add_root_node(PathBuf::from("main.c"), c_file.clone())?;
    let add_c = g.add_root_node(PathBuf::from("add.c"), c_file.clone())?;

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
    //     PathBuf::from("main.exe"),
    //     exe_file.clone(),
    //     // M::convert_fn(object_file_from_cfile),
    //     object_file_from_cfile,
    // )?;

    // g.add_edge(main_o, main_c)?;
    // g.add_edge(add_o, add_c)?;
    // g.add_edge(exe, main_o)?;
    // g.add_edge(exe, add_o)?;

    let basic_dot = Dot::new(&g.g);

    let pdot = PathBuf::from("out.dot");
    std::fs::write(pdot, format!("{:?}", basic_dot))?;

    log::info!("main foo");
    make(&g, PathBuf::from(cli.sandbox), cli.force, cli.nb_workers).await?;
    Ok(())
}
