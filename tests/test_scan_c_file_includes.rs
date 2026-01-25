//! Test C file header dependency scanning.

use std::path::PathBuf;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::GNode;

/// Tests that scanning a C file correctly detects `#include` directives.
///
/// The `scan()` method on `OFile` parses its predecessor C files to find
/// `#include "..."` directives and returns the paths to those header files.
/// This test verifies that `add.c` which includes `add.h` is correctly scanned.
#[test]
fn test_scan_c_file_includes() {
    let srcdir = PathBuf::from("demo_projects");

    // Create the nodes
    let add_c = CFile::new("project_1/add.c");
    let add_o = OFile::new("project_1/add.o", vec![], vec![]);

    // Create a boxed predecessor and get a trait object reference
    let add_c_box: Box<dyn GNode + Send + Sync> = Box::new(add_c);
    let predecessors: Vec<&(dyn GNode + Send + Sync)> = vec![add_c_box.as_ref()];

    // Call scan - reads from srcdir
    let result = add_o.scan(&srcdir, &predecessors);

    assert_eq!(result, vec![PathBuf::from("project_1/add.h")]);
}
