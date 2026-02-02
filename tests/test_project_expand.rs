//! Test building a project with expand functionality.
//!
//! This test verifies that the expand method works correctly to dynamically
//! generate nodes and edges during the build process.

mod common;

use common::{JsonDesc, YmlDesc};
use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;

#[test]
fn test_project_expand() {
    let srcdir = PathBuf::from("demo_projects");
    let sandbox = TempDir::new("yamake_test_expand").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path.clone());

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

    let result = g.make();

    assert!(result, "make should return true on successful build");

    // Verify the executable was built
    let app_path = sandbox_path.join("project_expand/app");
    assert!(app_path.exists(), "Executable should exist at {app_path:?}");

    // Verify the library was built
    let lib_path = sandbox_path.join("project_expand/generated/liblangs.a");
    assert!(lib_path.exists(), "Library should exist at {lib_path:?}");

    // Run the executable and check output
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the built executable");

    assert!(
        output.status.success(),
        "Executable should run successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, World!"),
        "Output should contain 'Hello, World!'"
    );
    assert!(
        stdout.contains("Bonjour"),
        "Output should contain 'Bonjour'"
    );
}
