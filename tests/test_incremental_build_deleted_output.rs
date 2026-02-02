//! Test incremental build after deleting an output file.

use std::fs;
use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::{G, GNodeStatus};

/// Tests that deleting an output file and rebuilding results in BuildNotChanged.
///
/// After first successful build, delete add.o from sandbox and rebuild:
/// - add.o should be rebuilt but have status BuildNotChanged (digest unchanged)
#[test]
fn test_incremental_build_after_delete() {
    let srcdir = PathBuf::from("demo_projects");
    let srcdir_abs = srcdir.canonicalize().expect("demo_projects should exist");
    // Use absolute path for external dependencies outside the project
    let include_paths = vec![srcdir_abs.join("other-deps/foo/bar")];

    let sandbox = TempDir::new("yamake_test_delete").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path.clone());

    let main_c = g.add_root_node(CFile::new("project_C/main.c")).unwrap();
    let main_o = g
        .add_node(OFile::new(
            "project_C/main.o",
            include_paths.clone(),
            vec![],
        ))
        .unwrap();
    let add_c = g.add_root_node(CFile::new("project_C/add.c")).unwrap();
    let add_o = g
        .add_node(OFile::new(
            "project_C/add.o",
            include_paths.clone(),
            vec!["-DYYY_defined".to_string()],
        ))
        .unwrap();
    let _add_h = g.add_root_node(HFile::new("project_C/add.h")).unwrap();
    let _wrapper_h = g.add_root_node(HFile::new("project_C/wrapper.h")).unwrap();
    let project_a = g.add_node(AFile::new("project_C/libproject.a")).unwrap();
    let app = g.add_node(XFile::new("project_C/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, project_a);
    g.add_edge(project_a, app);

    // First build - should succeed
    let result = g.make();
    assert!(result, "First build should succeed");

    // Verify add.o exists
    let add_o_path = sandbox_path.join("project_C/add.o");
    assert!(add_o_path.exists(), "add.o should exist after first build");

    // Delete add.o from sandbox
    fs::remove_file(&add_o_path).unwrap();
    assert!(!add_o_path.exists(), "add.o should be deleted");

    // Second build - should succeed and rebuild add.o
    let result = g.make();
    assert!(result, "Second build should succeed");

    // Verify add.o was rebuilt
    assert!(add_o_path.exists(), "add.o should exist after second build");

    // Check add.o is BuildNotChanged (rebuilt but digest unchanged)
    let status = g.nodes_status.get(&add_o);
    assert_eq!(
        status,
        Some(&GNodeStatus::BuildNotChanged),
        "add.o should be BuildNotChanged, got {status:?}"
    );
}
