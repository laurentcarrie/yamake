//! Test that scanning returns header paths even when they don't exist yet.

use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::GNode;

/// Tests that scanning returns header paths even for non-existent headers.
///
/// When a C file contains `#include "foo.h"` but `foo.h` doesn't exist,
/// the scan still returns foo.h in the result. This allows generated
/// headers (created by expand) to be handled correctly - the dependency
/// is recorded before the file exists.
#[test]
fn test_scan_returns_missing_header() {
    let sandbox = TempDir::new("yamake_scan_missing").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    // Create project_C directory in sandbox
    std::fs::create_dir_all(sandbox_path.join("project_C")).unwrap();

    // Create a C file that includes a non-existent header
    let add_c_content = "#include \"foo.h\"\nint add(int a, int b) { return a + b; }";
    std::fs::write(sandbox_path.join("project_C/add.c"), add_c_content).unwrap();

    // Create the nodes
    let add_c = CFile::new("project_C/add.c");
    let add_o = OFile::new("project_C/add.o", vec![], vec![]);

    // Create a boxed predecessor and get a trait object reference
    let add_c_box: Box<dyn GNode + Send + Sync> = Box::new(add_c);
    let predecessors: Vec<&(dyn GNode + Send + Sync)> = vec![add_c_box.as_ref()];

    // Call scan - returns the header path even though it doesn't exist
    let (scan_complete, result) = add_o.scan(&sandbox_path, &predecessors);

    // Scan is incomplete because the header doesn't exist
    assert!(!scan_complete, "Scan should be incomplete for missing header");
    // The missing header is still returned in the scan result
    assert_eq!(result, vec![PathBuf::from("foo.h")]);
}
