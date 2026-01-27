//! Test that scanning returns header paths as specified in include directives.

use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::GNode;

/// Tests that scan returns the exact path from the include directive.
///
/// If a C file uses `#include "add.h"` but the header is actually at `project_C/add.h`,
/// the scan returns "add.h" as specified. The build step will later fail if the
/// include path is wrong. This allows for flexible header discovery - headers may
/// be in include paths (-I flags) or generated later.
#[test]
fn test_scan_returns_include_path_as_specified() {
    let sandbox = TempDir::new("yamake_scan_path").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    // Create project_C directory in sandbox
    std::fs::create_dir_all(sandbox_path.join("project_C")).unwrap();

    // Create add.h in project_C (not at root)
    std::fs::write(
        sandbox_path.join("project_C/add.h"),
        "int add(int a, int b);",
    )
    .unwrap();

    // Create a C file that includes "add.h" without the project_C prefix
    let add_c_content = "#include \"add.h\"\nint add(int a, int b) { return a + b; }";
    std::fs::write(sandbox_path.join("project_C/add.c"), add_c_content).unwrap();

    // Create the nodes
    let add_c = CFile::new("project_C/add.c");
    let add_o = OFile::new("project_C/add.o", vec![], vec![]);

    // Create a boxed predecessor and get a trait object reference
    let add_c_box: Box<dyn GNode + Send + Sync> = Box::new(add_c);
    let predecessors: Vec<&(dyn GNode + Send + Sync)> = vec![add_c_box.as_ref()];

    // Call scan - returns the path as specified in the include directive
    let (scan_complete, result) = add_o.scan(&sandbox_path, &predecessors);

    // Scan is incomplete because add.h doesn't exist at the root (only at project_C/add.h)
    assert!(!scan_complete, "Scan should be incomplete for wrong include path");
    // Returns "add.h" as specified in the include, not "project_C/add.h"
    assert_eq!(result, vec![PathBuf::from("add.h")]);
}
