// ANCHOR: use
use argh::FromArgs;
use log::info;
use std::path::PathBuf;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;
// ANCHOR_END: use

// ANCHOR: use_existing_rules
// Using existing C build rules from yamake::c_nodes:
// - CFile: C source files (.c)
// - HFile: Header files (.h)
// - OFile: Object files (.o)
// - AFile: Static library files (.a)
// - XFile: Executable files
// ANCHOR_END: use_existing_rules

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

    let srcdir=args.src ;
    let sandbox=args.sandbox ;

    // ANCHOR: instanciate
    let mut g = G::new(srcdir, sandbox);
    // ANCHOR_END: instanciate

    // ANCHOR: add_nodes
    let main_c = g.add_root_node(CFile::new("project_1/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_1/main.o", vec![], vec![])).unwrap();
    let add_c = g.add_root_node(CFile::new("project_1/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_1/add.o", vec![], vec!["-DYYY_defined".to_string()])).unwrap();
    let _add_h = g.add_root_node(HFile::new("project_1/add.h")).unwrap();
    let _wrapper_h = g.add_root_node(HFile::new("project_1/wrapper.h")).unwrap();
    let project_a = g.add_node(AFile::new("project_1/libproject.a")).unwrap();
    let app = g.add_node(XFile::new("project_1/app")).unwrap();
    // ANCHOR_END: add_nodes

    // ANCHOR: add_edges
    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, project_a);
    g.add_edge(project_a, app);
    // ANCHOR_END: add_edges

    info!(
        "Created graph with {} nodes and {} edges",
        g.g.node_count(),
        g.g.edge_count()
    );

    // ANCHOR: make
    // make() will:
    // 1. Mount root nodes (copy source files to sandbox)
    // 2. Scan for dependencies (adds edges for #include directives)
    // 3. Build all nodes in dependency order (parallel where possible)
    let success = g.make();
    info!("Build {}", if success { "succeeded" } else { "failed" });
    // ANCHOR_END: make
}
