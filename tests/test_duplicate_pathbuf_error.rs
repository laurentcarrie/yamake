//! Test that duplicate node paths are rejected.

mod common;

use common::TargetFile;
use std::path::PathBuf;
use yamake::model::{G, GraphError};

/// Tests that adding two nodes with the same output path returns a `DuplicatePathBuf` error.
///
/// Each node's output path must be unique within a graph to prevent build conflicts.
/// This test verifies that attempting to add a second node with an existing path
/// fails with the appropriate error, even when the node IDs are different.
#[test]
fn test_duplicate_pathbuf_error() {
    let mut g = G::new(PathBuf::from("src"), PathBuf::from("build"));

    g.add_node(TargetFile {
        name: "node1".to_string(),
        path: PathBuf::from("same/path.txt"),
    })
    .unwrap();

    let result = g.add_node(TargetFile {
        name: "node2".to_string(),
        path: PathBuf::from("same/path.txt"),
    });

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GraphError::DuplicatePathBuf(_)
    ));
}
