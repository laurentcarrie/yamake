//! Test incremental build with a compilation failure.

use std::fs;
use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::{G, GNodeStatus};

/// Tests that modifying a header to introduce a syntax error causes correct failure propagation.
///
/// After first successful build, add 'xxx' to add.h which causes compilation failure:
/// - add.o should be BuildFailed (compilation fails)
/// - app should be AncestorFailed (depends on failed node)
#[test]
fn test_incremental_build_with_failure() {
    // Create temp directories for srcdir and sandbox
    let srcdir_temp = TempDir::new("yamake_test_srcdir").unwrap();
    let sandbox = TempDir::new("yamake_test_sandbox").unwrap();
    let srcdir = srcdir_temp.path().to_path_buf();
    let sandbox_path = sandbox.path().to_path_buf();

    // Copy demo_projects/project_C to temp srcdir
    let src_project = PathBuf::from("demo_projects/project_C");
    let dst_project = srcdir.join("project_C");
    fs::create_dir_all(&dst_project).unwrap();
    for entry in fs::read_dir(&src_project).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst_project.join(entry.file_name());
        fs::copy(&src_path, &dst_path).unwrap();
    }

    // Copy other-deps for project_C2/foo.h
    let other_deps_src = PathBuf::from("demo_projects/other-deps");
    let other_deps_dst = srcdir.join("other-deps");
    fs::create_dir_all(other_deps_dst.join("foo/bar/project_C2")).unwrap();
    fs::copy(
        other_deps_src.join("foo/bar/project_C2/foo.h"),
        other_deps_dst.join("foo/bar/project_C2/foo.h"),
    )
    .unwrap();

    // Use absolute path for external dependencies outside the project
    let include_paths = vec![srcdir.join("other-deps/foo/bar")];

    let mut g = G::new(srcdir.clone(), sandbox_path);

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

    // Modify add.h to introduce a syntax error
    let add_h_path = srcdir.join("project_C/add.h");
    let content = fs::read_to_string(&add_h_path).unwrap();
    fs::write(&add_h_path, format!("{content}\nxxx\n")).unwrap();

    // Second build - should fail
    let result = g.make();
    assert!(!result, "Second build should fail");

    // Check add.o is BuildFailed
    let status = g.nodes_status.get(&add_o);
    assert_eq!(
        status,
        Some(&GNodeStatus::BuildFailed),
        "add.o should be BuildFailed, got {status:?}"
    );

    // Check app is AncestorFailed
    let status = g.nodes_status.get(&app);
    assert_eq!(
        status,
        Some(&GNodeStatus::AncestorFailed),
        "app should be AncestorFailed, got {status:?}"
    );
}
