//! Test building a C project with headers found via -I include path.

use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, OFile, XFile};
use yamake::model::G;

/// Tests that the build succeeds when headers are found via -I include path.
///
/// Instead of declaring HFile nodes, this test uses the -I compile flag to
/// point gcc directly at the source directory where headers reside. This
/// allows building without mounting header files to the sandbox.
#[test]
fn test_build_with_include_path() {
    let srcdir = PathBuf::from("demo_projects");
    let srcdir_abs = srcdir.canonicalize().expect("demo_projects should exist");
    // This test doesn't add HFile nodes, so use srcdir_abs for project headers
    // and other-deps for external dependencies
    let include_paths = vec![srcdir_abs.clone(), srcdir_abs.join("other-deps/foo/bar")];

    let sandbox = TempDir::new("yamake_test_include_path").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path.clone());

    // Add only C files - no HFile nodes
    // Use include_paths to find headers directly from source directory
    let main_c = g.add_root_node(CFile::new("project_C/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_C/main.o", include_paths.clone(), vec![])).unwrap();
    let add_c = g.add_root_node(CFile::new("project_C/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_C/add.o", include_paths.clone(), vec!["-DYYY_defined".to_string()])).unwrap();
    // Note: No HFile nodes - headers are found via -I flag
    let project_a = g.add_node(AFile::new("project_C/libproject.a")).unwrap();
    let app = g.add_node(XFile::new("project_C/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, project_a);
    g.add_edge(project_a, app);

    // Only 6 nodes (no add.h, no wrapper.h)
    assert_eq!(g.g.node_count(), 6);
    assert_eq!(g.g.edge_count(), 5);

    // Build should succeed because headers are found via -I flag
    let result = g.make();
    assert!(result, "make should return true when headers are found via -I");

    // Verify the executable was built
    let app_path = sandbox_path.join("project_C/app");
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
