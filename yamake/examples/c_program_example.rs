use argh::FromArgs;
use log;
use simple_logging;
// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use walkdir::WalkDir;

use petgraph::dot::Dot;
use yamake::model as M;
use yamake::run::make;

use yamake::c_build::{c_file_scan, exe_from_obj_files, object_file_from_cfile};

pub struct CSource;

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

// fn c_scan(include_paths: Vec<PathBuf>) -> impl M::ScanFn {
//     |srcdir, target, stdout, stderr| c_file_scan(srcdir, target, stdout, stderr, include_paths)
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logging::log_to_file("make.log", log::LevelFilter::Info)?;

    let cli: Cli = argh::from_env();

    let srcdir = PathBuf::from(cli.srcdir);
    let sandbox = PathBuf::from(cli.sandbox);

    let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;

    let include_paths = vec![srcdir.clone()];

    // as in a Makefile, populate with the list of c file sources.
    // we also add the .h
    // populate the graph with source code.
    // in this demo, we walk in the srcdir and add all the .c and .h files
    for entry in WalkDir::new(srcdir).into_iter() {
        if let Ok(entry) = entry
            && entry.path().is_file()
        {
            if let Some(s) = entry.path().extension() {
                match s.to_str() {
                    Some("c") => g.add_root_node(
                        entry.path().to_path_buf(),
                        "c file".to_string(),
                        c_file_scan,
                    )?,
                    Some("h") => g.add_root_node(
                        entry.path().to_path_buf(),
                        "h file".to_string(),
                        c_file_scan,
                    )?,
                    s => log::info!("ignored entry : {:?}", s),
                }
            }
        }
    }

    // as in Makefile where .c.o is an implicit rule,
    // for each .c file we add a .o file the explicit dependecy and the build function
    for (source, _nid) in g.map.clone() {
        if source.extension().map(|x| x.to_str()).flatten() == Some("c") {
            let target = source.with_extension("o");
            g.add_node(target.clone(), "o file".to_string(), object_file_from_cfile)?;
            g.add_edge(target.clone(), target.clone().with_extension("c"))?;
        }
    }

    // as in a Makefile, explicit how your executable depends on sources
    let exe = PathBuf::from("project_1/demo");
    g.add_node(exe.clone(), "x file".to_string(), exe_from_obj_files)?;
    g.add_edge(exe.clone(), PathBuf::from("project_1/main.o"))?;
    g.add_edge(exe.clone(), PathBuf::from("project_1/add.o"))?;

    // add scanned dependencies
    match g.scan().await {
        Ok(()) => (),
        Err(e) => {
            println!("{}", e.to_string());
            std::process::exit(1)
        }
    }

    // for demo or debug, output the tree
    let basic_dot = Dot::new(&g.g);
    let pdot = PathBuf::from("out.dot");
    std::fs::write(pdot, format!("{:?}", basic_dot))?;

    log::info!("dot file written");

    log::info!("main foo");
    match g.make(cli.force, cli.nb_workers).await {
        Ok(()) => (),
        Err(e) => println!("{}", e.to_string()),
    };
    Ok(())
}
