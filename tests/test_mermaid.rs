//! Test Mermaid graph generation.

use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{CFile, HFile, OFile, XFile};
use yamake::model::G;

#[test]
fn test_to_mermaid() {
    let srcdir = PathBuf::from("demo_projects");
    let sandbox = TempDir::new("yamake_mermaid_test").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path);

    let main_c = g.add_root_node(CFile::new("main.c")).unwrap();
    let main_o = g.add_node(OFile::new("main.o", vec![], vec![])).unwrap();
    let _main_h = g.add_root_node(HFile::new("main.h")).unwrap();
    let app = g.add_node(XFile::new("app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);

    let mermaid = g.to_mermaid();

    // Check basic structure
    assert!(
        mermaid.contains("flowchart LR"),
        "Should have flowchart header"
    );
    assert!(mermaid.contains("CFile"), "Should contain CFile tag");
    assert!(mermaid.contains("OFile"), "Should contain OFile tag");
    assert!(mermaid.contains("XFile"), "Should contain XFile tag");
    assert!(mermaid.contains("main.c"), "Should contain main.c filename");
    assert!(mermaid.contains("-->"), "Should contain edges");

    println!("Generated Mermaid:\n{}", mermaid);
}
