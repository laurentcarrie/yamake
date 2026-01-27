//! Test C file header dependency scanning.

use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::GNode;

/// Tests that scanning a C file correctly detects `#include` directives.
///
/// The `scan()` method on `OFile` parses its predecessor C files to find
/// `#include "..."` directives and returns the paths to those header files.
/// This test verifies that `add.c` which includes `add.h` is correctly scanned.
#[test]
fn test_scan_c_file_includes() {
    let sandbox = TempDir::new("yamake_scan_test").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    // Create project_C directory in sandbox
    std::fs::create_dir_all(sandbox_path.join("project_C")).unwrap();

    // Create add.c that includes add.h
    let add_c_content = "#include \"project_C/add.h\"\nint add(int a, int b) { return a + b; }";
    std::fs::write(sandbox_path.join("project_C/add.c"), add_c_content).unwrap();

    // Create add.h
    std::fs::write(
        sandbox_path.join("project_C/add.h"),
        "int add(int a, int b);",
    )
    .unwrap();

    // Create the nodes
    let add_c = CFile::new("project_C/add.c");
    let add_o = OFile::new("project_C/add.o", vec![], vec![]);

    // Create a boxed predecessor and get a trait object reference
    let add_c_box: Box<dyn GNode + Send + Sync> = Box::new(add_c);
    let predecessors: Vec<&(dyn GNode + Send + Sync)> = vec![add_c_box.as_ref()];

    // Call scan - reads from sandbox
    let (scan_complete, result) = add_o.scan(&sandbox_path, &predecessors);

    assert!(scan_complete, "Scan should be complete");
    assert_eq!(result, vec![PathBuf::from("project_C/add.h")]);
}
