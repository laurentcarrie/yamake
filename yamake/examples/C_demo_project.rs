use argh::FromArgs;
use log;
// use petgraph::Direction::Incoming;
use simple_logging;
// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;

use petgraph::dot::Dot;

// ANCHOR: use
use yamake::model as M;
// ANCHOR_END: use

// ANCHOR: use_existing_rules
// .c source file
use yamake::rules::c_rules::c_file::Cfile;
// .h source file
use yamake::rules::c_rules::h_file::Hfile;
// .o built object file
use yamake::rules::c_rules::o_file::Ofile;
// executeble built file
use yamake::rules::c_rules::x_file::Xfile;

use yamake::rules::c_rules;

// granted, we should add .a libraries and .so dynamic link libraries
// ANCHOR_END: use_existing_rules

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

    // ANCHOR: instanciate

    let cli: Cli = argh::from_env();

    let srcdir = PathBuf::from(cli.srcdir)
        .canonicalize()
        .expect("canonicalize srcdir");
    let sandbox = PathBuf::from(cli.sandbox)
        .canonicalize()
        .expect("canonicalize sandbox");

    let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;

    // ANCHOR_END: instanciate

    // don't use the srcdir ! we take the header files from the sandbox
    // take everything from sandbox, there might be generated files

    // ANCHOR: add_nodes
    for f in vec!["project_1/main.c", "project_1/add.c"] {
        g.add_node(Cfile::new(f.into())?)?;
    }
    for f in vec!["project_1/add.h", "project_1/wrapper.h"] {
        g.add_node(Hfile::new(f.into())?)?;
    }

    g.add_node(c_rules::yml_language::F::new(
        "project_1/greetings.yml".into(),
    )?)?;

    let include_paths = vec![sandbox.clone()];
    let compile_options = vec!["-Wall".into(), "-O2".into()];
    for f in vec!["project_1/main.o", "project_1/add.o"] {
        g.add_node(Ofile::new(
            f.into(),
            include_paths.clone(),
            compile_options.clone(),
        )?)?;
    }

    let link_flags = vec![];
    g.add_node(Xfile::new("project_1/demo".into(), link_flags)?)?;

    // ANCHOR_END: add_nodes

    // ANCHOR: add_edges
    g.add_edge("project_1/main.o".into(), "project_1/main.c".into())?;
    g.add_edge("project_1/add.o".into(), "project_1/add.c".into())?;
    g.add_edge("project_1/demo".into(), "project_1/main.o".into())?;
    g.add_edge("project_1/demo".into(), "project_1/add.o".into())?;

    // ANCHOR_END: add_edges

    // ANCHOR: before-scan
    let basic_dot = Dot::new(&g.g);
    let mut pdot = sandbox.clone();
    pdot.push("before-scan.dot");
    std::fs::write(pdot, format!("{:?}", basic_dot))?;
    // ANCHOR_END: before-scan

    // ANCHOR: make
    match g.make(cli.force, cli.nb_workers).await {
        Ok(ret) => {
            println!("success : {}", ret.success);
            // you can walk the graph and print status of each node
            // for (k, v) in ret.nt {
            //     println!("node {:?} : {:?}", k, v);
            // }
        }

        Err(e) => println!("{}", e.to_string()),
    };
    // ANCHOR_END: make

    // write_current_hash(&g)?;
    Ok(())
}
