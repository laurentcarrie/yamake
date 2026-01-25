//! Test that scanning fails when include paths don't match file locations.

use tempdir::TempDir;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::GNode;

/// Tests that scanning panics when an include path doesn't match the actual file location.
///
/// If a C file uses `#include "add.h"` but the header is actually at `project_1/add.h`,
/// the scan should fail because include paths are resolved relative to the source
/// directory root, not relative to the including file's directory.
#[test]
#[should_panic(expected = "Header file not found: add.h")]
fn test_scan_fails_on_wrong_include_path() {
    let srcdir = TempDir::new("yamake_scan_wrong_path").unwrap();
    let srcdir_path = srcdir.path().to_path_buf();

    // Create project_1 directory in temp srcdir
    std::fs::create_dir_all(srcdir_path.join("project_1")).unwrap();

    // Create add.h in project_1 (correct location)
    std::fs::write(
        srcdir_path.join("project_1/add.h"),
        "int add(int a, int b);",
    )
    .unwrap();

    // Create a C file that includes "add.h" without the project_1 prefix
    let add_c_content = "#include \"add.h\"\nint add(int a, int b) { return a + b; }";
    std::fs::write(srcdir_path.join("project_1/add.c"), add_c_content).unwrap();

    // Create the nodes
    let add_c = CFile::new("project_1/add.c");
    let add_o = OFile::new("project_1/add.o");

    // Create a boxed predecessor and get a trait object reference
    let add_c_box: Box<dyn GNode + Send + Sync> = Box::new(add_c);
    let predecessors: Vec<&(dyn GNode + Send + Sync)> = vec![add_c_box.as_ref()];

    // Call scan - should panic because "add.h" doesn't exist at srcdir/add.h
    add_o.scan(&srcdir_path, &predecessors);
}
