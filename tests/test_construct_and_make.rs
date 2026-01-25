//! Test basic graph construction and the make operation.

mod common;

use common::{SourceFile, TargetFile};
use std::path::PathBuf;
use yamake::model::G;

/// Tests that a simple graph with one source and one target node can be
/// constructed and built using `make()`.
///
/// This test verifies:
/// - Graph creation with source and sandbox directories
/// - Adding root nodes (source files)
/// - Adding target nodes (build outputs)
/// - Creating edges between nodes
/// - Running the make operation
#[test]
fn test_construct_and_make() {
    let mut g = G::new(PathBuf::from("src"), PathBuf::from("build"));

    let src = g
        .add_root_node(SourceFile {
            name: "input.txt".to_string(),
        })
        .unwrap();
    let target = g
        .add_node(TargetFile {
            path: PathBuf::from("output.txt"),
        })
        .unwrap();

    g.add_edge(src, target);

    assert_eq!(g.g.node_count(), 2);
    assert_eq!(g.g.edge_count(), 1);

    g.make();
}
