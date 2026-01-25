//! Test that scanning fails when a referenced header file doesn't exist.

use tempdir::TempDir;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::GNode;

/// Tests that scanning panics when a C file includes a non-existent header.
///
/// When a C file contains `#include "foo.h"` but `foo.h` doesn't exist in the
/// source directory, the scan operation should fail early with a clear error
/// message rather than proceeding with an incomplete dependency graph.
#[test]
#[should_panic(expected = "Header file not found: foo.h")]
fn test_scan_fails_on_missing_header() {
    let srcdir = TempDir::new("yamake_scan_fail_srcdir").unwrap();
    let srcdir_path = srcdir.path().to_path_buf();

    // Create project_1 directory in temp srcdir
    std::fs::create_dir_all(srcdir_path.join("project_1")).unwrap();

    // Create a C file that includes a non-existent header
    let add_c_content = "#include \"foo.h\"\nint add(int a, int b) { return a + b; }";
    std::fs::write(srcdir_path.join("project_1/add.c"), add_c_content).unwrap();

    // Create the nodes
    let add_c = CFile::new("project_1/add.c");
    let add_o = OFile::new("project_1/add.o", vec![], vec![]);

    // Create a boxed predecessor and get a trait object reference
    let add_c_box: Box<dyn GNode + Send + Sync> = Box::new(add_c);
    let predecessors: Vec<&(dyn GNode + Send + Sync)> = vec![add_c_box.as_ref()];

    // Call scan - should panic because foo.h doesn't exist
    add_o.scan(&srcdir_path, &predecessors);
}
