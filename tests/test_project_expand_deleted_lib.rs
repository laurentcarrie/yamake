//! Test incremental build after deleting the static library.
//!
//! This test verifies that deleting liblangs.a triggers a rebuild.

mod common;

use common::{JsonDesc, YmlDesc};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;

/// Helper to recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

/// Helper to create the graph
fn create_graph(srcdir: &Path, sandbox_path: &Path) -> G {
    let mut g = G::new(srcdir.to_path_buf(), sandbox_path.to_path_buf());

    let main_c = g
        .add_root_node(CFile::new("project_expand/main.c"))
        .unwrap();
    let main_o = g
        .add_node(OFile::new("project_expand/main.o", vec![], vec![]))
        .unwrap();
    let _wrapper_h = g
        .add_root_node(HFile::new("project_expand/wrapper.h"))
        .unwrap();
    let app = g.add_node(XFile::new("project_expand/app")).unwrap();

    let languages_yml = g
        .add_root_node(YmlDesc::new("project_expand/languages.yml"))
        .unwrap();
    let languages_json = g
        .add_node(JsonDesc::new("project_expand/languages.json"))
        .unwrap();
    let liblangs = g
        .add_node(AFile::new("project_expand/generated/liblangs.a"))
        .unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(languages_yml, languages_json);
    g.add_edge(languages_json, liblangs);
    g.add_edge(liblangs, app);

    g
}

/// Tests that deleting liblangs.a triggers a rebuild.
#[test]
fn test_project_expand_deleted_lib() {
    // Create temp directories for srcdir and sandbox
    let srcdir_temp = TempDir::new("yamake_expand_srcdir").unwrap();
    let sandbox = TempDir::new("yamake_expand_sandbox").unwrap();
    let srcdir = srcdir_temp.path().to_path_buf();
    let sandbox_path = sandbox.path().to_path_buf();

    // Copy demo_projects/project_expand to temp srcdir
    let src_project = PathBuf::from("demo_projects/project_expand");
    let dst_project = srcdir.join("project_expand");
    copy_dir_recursive(&src_project, &dst_project);

    // First build - should succeed
    let mut g = create_graph(&srcdir, &sandbox_path);
    let result = g.make();
    assert!(result, "First build should succeed");

    // Verify the library exists
    let lib_path = sandbox_path.join("project_expand/generated/liblangs.a");
    assert!(lib_path.exists(), "Library should exist after first build");

    // Verify the executable works
    let app_path = sandbox_path.join("project_expand/app");
    assert!(
        app_path.exists(),
        "Executable should exist at {:?}",
        app_path
    );
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the built executable");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, World!"),
        "First build output should contain 'Hello, World!'"
    );

    // Delete the static library
    fs::remove_file(&lib_path).expect("Failed to delete liblangs.a");
    assert!(!lib_path.exists(), "Library should be deleted");

    // Second build - should rebuild the library
    let mut g = create_graph(&srcdir, &sandbox_path);
    let result = g.make();
    assert!(result, "Second build should succeed");

    // Verify the library was rebuilt
    assert!(lib_path.exists(), "Library should exist after second build");

    // Verify the executable still works
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the rebuilt executable");

    assert!(
        output.status.success(),
        "Executable should run successfully after rebuild"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, World!"),
        "Second build output should contain 'Hello, World!'"
    );
    assert!(
        stdout.contains("Bonjour"),
        "Second build output should contain 'Bonjour'"
    );
}
