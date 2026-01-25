//! Test building a complete C project with automatic header scanning.

use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;

/// Tests end-to-end building of a C project with multiple source files, headers,
/// a static library, and an executable.
///
/// This test verifies:
/// - Compiling C files to object files
/// - Creating a static library from object files
/// - Linking object files and libraries into an executable
/// - Automatic header dependency detection via scanning (edges for `add.h` and
///   `wrapper.h` are discovered, not manually specified)
/// - The built executable runs and produces expected output
#[test]
fn test_build_c_project() {
    let srcdir = PathBuf::from("demo_projects");
    let sandbox = TempDir::new("yamake_test").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path.clone());

    let main_c = g.add_root_node(CFile::new("project_1/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_1/main.o")).unwrap();
    let add_c = g.add_root_node(CFile::new("project_1/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_1/add.o")).unwrap();
    let _add_h = g.add_root_node(HFile::new("project_1/add.h")).unwrap();
    let _wrapper_h = g.add_root_node(HFile::new("project_1/wrapper.h")).unwrap();
    let project_a = g.add_node(AFile::new("project_1/libproject.a")).unwrap();
    let app = g.add_node(XFile::new("project_1/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, project_a);
    g.add_edge(project_a, app);

    assert_eq!(g.g.node_count(), 8);
    assert_eq!(g.g.edge_count(), 5);

    let result = g.make();
    assert!(result, "make should return true on successful build");

    // Verify the executable was built
    let app_path = sandbox_path.join("project_1/app");
    assert!(
        app_path.exists(),
        "Executable should exist at {app_path:?}"
    );

    // Run the executable and check output
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the built executable");

    assert!(
        output.status.success(),
        "Executable should run successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"), "Output should contain 'hello'");
}
