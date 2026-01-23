use yamake::model::G;
use yamake::c_nodes::{CFile, HFile, OFile, XFile};
use std::path::PathBuf;
use argh::FromArgs;

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
    let args: Args = argh::from_env();

    let mut g = G::new(args.src, args.sandbox);

    let main_c = g.add_root_node(CFile::new("project_1/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_1/main.o")).unwrap();
    let add_c = g.add_root_node(CFile::new("project_1/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_1/add.o")).unwrap();
    let add_h = g.add_root_node(HFile::new("project_1/add.h")).unwrap();
    let wrapper_h = g.add_root_node(HFile::new("project_1/wrapper.h")).unwrap();
    let app = g.add_node(XFile::new("project_1/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(wrapper_h, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_h, add_o);
    g.add_edge(add_o, app);

    println!("Created graph with {} nodes and {} edges",
        g.g.node_count(),
        g.g.edge_count());

    println!("\nWalking graph (DFS from app):");
    g.walk_graph(app);

    println!("\nBuilding graph (DFS from app):");
    g.build_graph(app);
}
