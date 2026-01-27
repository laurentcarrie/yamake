//! Test incremental build with expand functionality.
//!
//! This test verifies that adding a new language to languages.yml triggers
//! a rebuild that includes the new language in the output.
//!
//! When the graph is recreated between builds, files discovered by scan that
//! exist in the sandbox (from previous builds) are automatically tracked as
//! PlaceholderFile nodes, enabling correct incremental rebuild detection.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;
use common::{JsonDesc, Language, YmlDesc};

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

    let main_c = g.add_root_node(CFile::new("project_expand/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_expand/main.o", vec![], vec![])).unwrap();
    let _wrapper_h = g.add_root_node(HFile::new("project_expand/wrapper.h")).unwrap();
    let app = g.add_node(XFile::new("project_expand/app")).unwrap();

    let languages_yml = g.add_root_node(YmlDesc::new("project_expand/languages.yml")).unwrap();
    let languages_json = g.add_node(JsonDesc::new("project_expand/languages.json")).unwrap();
    let liblangs = g.add_node(AFile::new("project_expand/generated/liblangs.a")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(languages_yml, languages_json);
    g.add_edge(languages_json, liblangs);
    g.add_edge(liblangs, app);

    g
}

/// Tests that adding a new language to languages.yml triggers a rebuild with the new language.
///
/// After first successful build with 5 languages, add Polish and rebuild:
/// - The rebuild should succeed
/// - The output should contain the Polish hello world
#[test]
fn test_project_expand_add_language() {
    // Create temp directories for srcdir and sandbox
    let srcdir_temp = TempDir::new("yamake_expand_srcdir").unwrap();
    let sandbox = TempDir::new("yamake_expand_sandbox").unwrap();
    let srcdir = srcdir_temp.path().to_path_buf();
    let sandbox_path = sandbox.path().to_path_buf();

    // Copy demo_projects/project_expand to temp srcdir
    let src_project = PathBuf::from("demo_projects/project_expand");
    let dst_project = srcdir.join("project_expand");
    copy_dir_recursive(&src_project, &dst_project);

    // First build - should succeed with 5 languages
    let mut g = create_graph(&srcdir, &sandbox_path);
    let result = g.make();
    assert!(result, "First build should succeed");

    // Verify initial output doesn't contain Polish
    let app_path = sandbox_path.join("project_expand/app");
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the built executable");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Witaj"), "Initial output should not contain Polish");

    // Add Polish language to languages.yml
    let languages_yml_path = srcdir.join("project_expand/languages.yml");
    let content = fs::read_to_string(&languages_yml_path).unwrap();
    let mut languages: Vec<Language> = serde_yaml::from_str(&content).unwrap();
    languages.push(Language {
        language: "Polish".to_string(),
        helloworld: "Witaj, Świecie!".to_string(),
    });
    fs::write(&languages_yml_path, serde_yaml::to_string(&languages).unwrap()).unwrap();

    // Second build - should succeed and include Polish
    let mut g = create_graph(&srcdir, &sandbox_path);
    let result = g.make();
    assert!(result, "Second build should succeed");

    // Verify the executable was rebuilt with Polish
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the rebuilt executable");

    assert!(
        output.status.success(),
        "Executable should run successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"), "Output should still contain 'Hello, World!'");
    assert!(stdout.contains("Bonjour"), "Output should still contain 'Bonjour'");
    assert!(stdout.contains("Witaj"), "Output should contain Polish 'Witaj'");

    // Remove English from languages.yml
    let content = fs::read_to_string(&languages_yml_path).unwrap();
    let mut languages: Vec<Language> = serde_yaml::from_str(&content).unwrap();
    languages.retain(|lang| lang.language != "English");
    fs::write(&languages_yml_path, serde_yaml::to_string(&languages).unwrap()).unwrap();

    // Third build - should succeed without English
    let mut g = create_graph(&srcdir, &sandbox_path);
    let result = g.make();
    assert!(result, "Third build should succeed");

    // Verify the executable was rebuilt without English
    let output = Command::new(&app_path)
        .output()
        .expect("Failed to run the rebuilt executable");

    assert!(
        output.status.success(),
        "Executable should run successfully after removing English"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Hello, World!"), "Output should NOT contain 'Hello, World!' after removing English");
    assert!(stdout.contains("Bonjour"), "Output should still contain 'Bonjour'");
    assert!(stdout.contains("Witaj"), "Output should still contain Polish 'Witaj'");
}
