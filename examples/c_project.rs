use argh::FromArgs;
use log::info;
use std::path::PathBuf;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;

#[derive(FromArgs)]
/// A C project build example
struct Args {
    /// source directory
    #[argh(option, short = 's')]
    src: PathBuf,

    /// sandbox directory
    #[argh(option, short = 'b')]
    sandbox: PathBuf,
}

fn main() {
    env_logger::init();

    let args: Args = argh::from_env();

    let mut g = G::new(args.src, args.sandbox);

    let main_c = g.add_root_node(CFile::new("project_1/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_1/main.o")).unwrap();
    let add_c = g.add_root_node(CFile::new("project_1/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_1/add.o")).unwrap();
    let _add_h = g.add_root_node(HFile::new("project_1/add.h")).unwrap();
    let _wrapper_h = g.add_root_node(HFile::new("project_1/wrapper.h")).unwrap();
    let project_a = g.add_node(AFile::new("project_1/libproject.a")).unwrap();
    let app = g.add_node(XFile::new("project_1/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, project_a);
    g.add_edge(project_a, app);

    info!(
        "Created graph with {} nodes and {} edges",
        g.g.node_count(),
        g.g.edge_count()
    );

    info!("Building graph:");
    g.make();
}
