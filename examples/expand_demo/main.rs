use argh::FromArgs;
use log;
// use petgraph::Direction::Incoming;
// use tokio::process::Child;
use std::path::PathBuf;
// use tokio::process::Command;

use petgraph::dot::Dot;

use yamake::model as M;
use yamake::rules::c_rules as R;

mod greetings_yml;

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
    yamake::helpers::log::setup_logger(false, log::LevelFilter::Info)?;
    // ANCHOR: instanciate

    let cli: Cli = argh::from_env();

    let srcdir = PathBuf::from(cli.srcdir)
        .canonicalize()
        .expect("canonicalize srcdir");
    let sandbox = PathBuf::from(cli.sandbox)
        .canonicalize()
        .expect("canonicalize sandbox");

    let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;

    // don't use the srcdir ! we take the header files from the sandbox
    // take everything from sandbox, there might be generated files

    g.add_node(R::c_file::new("demo_expand/main.c".into())?)?;
    g.add_node(greetings_yml::N::new("demo_expand/greetings.yml".into())?)?;
    g.add_node(R::a_file::new("demo_expand/liblanguages.a".into(), vec![])?)?;

    let include_paths = vec![sandbox.clone()];
    let compile_options = vec!["-Wall".into(), "-O2".into()];
    g.add_node(R::o_file::new(
        "demo_expand/main.o".into(),
        include_paths.clone(),
        compile_options.clone(),
    )?)?;

    let link_flags = vec![];
    g.add_node(R::x_file::new("demo_expand/demo".into(), link_flags)?)?;

    g.add_edge("demo_expand/main.o".into(), "demo_expand/main.c".into())?;

    let basic_dot = Dot::new(&g.g);
    let mut pdot = sandbox.clone();
    pdot.push("before-scan.dot");
    std::fs::write(pdot, format!("{:?}", basic_dot))?;

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
