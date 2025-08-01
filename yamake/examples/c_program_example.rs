use argh::FromArgs;
use log;
use petgraph::Direction::Incoming;
use simple_logging;
// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;
use walkdir::WalkDir;

use petgraph::dot::Dot;
use yamake::model as M;

use yamake::c_project::c_file::Cfile;
use yamake::c_project::h_file::Hfile;
use yamake::c_project::o_file::Ofile;
use yamake::c_project::x_file::Xfile;

use yamake::target_hash::get_current_hash;

use petgraph::graph::NodeIndex;

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

    let srcdir = PathBuf::from(cli.srcdir)
        .canonicalize()
        .expect("canonicalize srcdir");
    let sandbox = PathBuf::from(cli.sandbox)
        .canonicalize()
        .expect("canonicalize sandbox");

    let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;

    let include_paths = vec![srcdir.clone()];
    // let include_paths = vec![];

    // as in a Makefile, populate with the list of c file sources.
    // we also add the .h
    // populate the graph with source code.
    // in this demo, we walk in the srcdir and add all the .c and .h files
    for entry in WalkDir::new(&srcdir).into_iter() {
        if let Ok(entry) = entry
            && entry.path().is_file()
        {
            let p = entry.path().to_path_buf().canonicalize()?;
            let p = p
                .strip_prefix(&srcdir)
                .expect(format!("strip {:?}", p).as_str())
                .to_path_buf();

            if let Some(s) = entry.path().extension() {
                match s.to_str() {
                    Some("c") => {
                        g.add_node(Cfile::new(p, include_paths.clone())?)?;
                        ()
                    }
                    Some("h") => {
                        g.add_node(Hfile::new(p, include_paths.clone())?)?;
                        ()
                    }
                    s => log::info!("ignored entry : {:?}", s),
                }
            }
        }
    }

    {
        // as in Makefile where .c.o is an implicit rule,

        // for each .c file we add a .o file the explicit dependecy and the build function
        // unimplemented!();

        let mut oc_files: Vec<(PathBuf, PathBuf)> = Vec::new();
        for ni in g.g.node_indices() {
            if g.g.edges_directed(ni, Incoming).count() == 0 {
                let n = g.g.node_weight(ni).ok_or("huh")?;
                if n.tag() == "c file" {
                    oc_files.push((n.target(), n.target().with_extension("o")));
                }
            }
        }
        for (c, o) in oc_files {
            g.add_node(Ofile::new(o.clone(), include_paths.clone())?)?;
            g.add_edge(o, c)?;
        }
    }

    // as in a Makefile, explicit how your executable depends on sources
    let exe = PathBuf::from("project_1/demo");
    g.add_node(Xfile::new(exe.clone())?)?;
    g.add_edge(exe.clone(), PathBuf::from("project_1/main.o"))?;
    g.add_edge(exe.clone(), PathBuf::from("project_1/add.o"))?;

    get_current_hash(&g)?;

    g.scan().await?;

    {
        let ni: NodeIndex = NodeIndex::new(0);
        let n = g.g.node_weight(ni).unwrap();
        let nn = n.clone();

        tokio::task::spawn(async move { log::info!("ZZZZ ; {:?}", nn.tag()) });
    }

    // tokio::task::spawn(async move { log::info!("ZZZZ ; {:?}", n.tag()) });

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
