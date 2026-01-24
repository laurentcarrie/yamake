//! Test that duplicate node IDs are rejected.

mod common;

use common::SourceFile;
use std::path::PathBuf;
use yamake::model::{G, GraphError};

/// Tests that adding two nodes with the same ID returns a `DuplicateId` error.
///
/// Node IDs must be unique within a graph. This test verifies that attempting
/// to add a second node with an existing ID fails with the appropriate error.
#[test]
fn test_duplicate_id_error() {
    let mut g = G::new(PathBuf::from("src"), PathBuf::from("build"));

    g.add_root_node(SourceFile {
        name: "input.txt".to_string(),
    })
    .unwrap();

    let result = g.add_root_node(SourceFile {
        name: "input.txt".to_string(),
    });

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GraphError::DuplicateId(_)));
}
