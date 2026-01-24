//! Test build failure handling and status propagation.

use tempdir::TempDir;
use yamake::c_nodes::{CFile, HFile, OFile, XFile};
use yamake::model::{G, GNodeStatus};

/// Tests that build failures are handled correctly and propagated to dependent nodes.
///
/// This test creates a C project with an invalid header file (syntax error) and verifies:
/// - `make()` returns `false` when a build fails
/// - The failing node receives `BuildFailed` status
/// - All nodes that depend on the failed node receive `AncestorFailed` status
/// - The final executable is not created when dependencies fail
#[test]
fn test_build_fails_with_invalid_header() {
    let srcdir = TempDir::new("yamake_build_fail").unwrap();
    let srcdir_path = srcdir.path().to_path_buf();
    let sandbox = TempDir::new("yamake_build_fail_sandbox").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    // Create project_1 directory
    std::fs::create_dir_all(srcdir_path.join("project_1")).unwrap();

    // Copy files from demo_projects/project_1
    std::fs::copy(
        "demo_projects/project_1/main.c",
        srcdir_path.join("project_1/main.c"),
    )
    .unwrap();
    std::fs::copy(
        "demo_projects/project_1/add.c",
        srcdir_path.join("project_1/add.c"),
    )
    .unwrap();
    std::fs::copy(
        "demo_projects/project_1/wrapper.h",
        srcdir_path.join("project_1/wrapper.h"),
    )
    .unwrap();

    // Copy add.h and append 'xzz' to cause a syntax error
    let add_h_content = std::fs::read_to_string("demo_projects/project_1/add.h").unwrap();
    std::fs::write(
        srcdir_path.join("project_1/add.h"),
        format!("{}xzz", add_h_content),
    )
    .unwrap();

    let mut g = G::new(srcdir_path, sandbox_path.clone());

    let main_c = g.add_root_node(CFile::new("project_1/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_1/main.o")).unwrap();
    let add_c = g.add_root_node(CFile::new("project_1/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_1/add.o")).unwrap();
    let _add_h = g.add_root_node(HFile::new("project_1/add.h")).unwrap();
    let _wrapper_h = g.add_root_node(HFile::new("project_1/wrapper.h")).unwrap();
    let app = g.add_node(XFile::new("project_1/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, app);

    let result = g.make();
    assert!(!result, "make should return false on build failure");

    // The executable should not exist due to build failure
    let app_path = sandbox_path.join("project_1/app");
    assert!(!app_path.exists(), "Executable should not exist due to build failure");

    // Check that at least one node has BuildFailed status
    let has_build_failed = g
        .nodes_status
        .values()
        .any(|&status| status == GNodeStatus::BuildFailed);
    assert!(has_build_failed, "At least one node should have BuildFailed status");

    // Check that app node has AncestorFailed status
    assert_eq!(
        g.nodes_status.get(&app),
        Some(&GNodeStatus::AncestorFailed),
        "app node should have AncestorFailed status"
    );
}
