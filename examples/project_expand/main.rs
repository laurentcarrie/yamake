#[path = "../../tests/common/mod.rs"]
mod common;

use argh::FromArgs;
use common::{JsonDesc, YmlDesc};
use log::info;
use std::fs;
use std::path::PathBuf;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;

#[derive(FromArgs)]
/// A C project build example demonstrating expand functionality
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

    let srcdir = args.src;
    let sandbox = args.sandbox;

    let mut g = G::new(srcdir.clone(), sandbox);

    let main_c = g
        .add_root_node(CFile::new("project_expand/main.c"))
        .unwrap();
    let main_o = g
        .add_node(OFile::new("project_expand/main.o", vec![], vec![]))
        .unwrap();
    let _wrapper_h = g
        .add_root_node(HFile::new("project_expand/wrapper.h"))
        .unwrap();
    let app = g.add_node(XFile::new("project_expand/app")).unwrap();

    let languages_yml = g
        .add_root_node(YmlDesc::new("project_expand/languages.yml"))
        .unwrap();
    let languages_json = g
        .add_node(JsonDesc::new("project_expand/languages.json"))
        .unwrap();
    let liblangs = g
        .add_node(AFile::new("project_expand/generated/liblangs.a"))
        .unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(languages_yml, languages_json);
    g.add_edge(languages_json, liblangs);
    g.add_edge(liblangs, app);

    info!(
        "Created graph with {} nodes and {} edges",
        g.g.node_count(),
        g.g.edge_count()
    );

    let success = g.make();
    info!("Build {}", if success { "succeeded" } else { "failed" });

    // Write mermaid graph to file
    let mermaid_path = srcdir.join("project_expand/graph.mermaid");
    let mermaid = g.to_mermaid();
    fs::write(&mermaid_path, &mermaid).expect("Failed to write graph.mermaid");
    info!("Wrote mermaid graph to {}", mermaid_path.display());
}
