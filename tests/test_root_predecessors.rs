//! Test the root_predecessors function.

use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{CFile, HFile, OFile, XFile};
use yamake::model::G;

/// Tests that root_predecessors returns the correct root nodes for a given node.
///
/// In the test graph:
/// - main.c -> main.o -> app
/// - add.c -> add.o -> app
/// - add.h -> add.o (via scan)
/// - wrapper.h -> main.o (via scan, includes add.h)
/// - add.h -> main.o (via scan, through wrapper.h)
///
/// For main.o, the root predecessors should be: main.c, wrapper.h, add.h
/// add.c should NOT be in the root predecessors of main.o
#[test]
fn test_root_predecessors_excludes_unrelated() {
    let srcdir = PathBuf::from("demo_projects");
    let sandbox = TempDir::new("yamake_test_root_pred").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path);

    let main_c = g.add_root_node(CFile::new("project_C/main.c")).unwrap();
    let main_o = g
        .add_node(OFile::new("project_C/main.o", vec![], vec![]))
        .unwrap();
    let add_c = g.add_root_node(CFile::new("project_C/add.c")).unwrap();
    let add_o = g
        .add_node(OFile::new(
            "project_C/add.o",
            vec![],
            vec!["-DYYY_defined".to_string()],
        ))
        .unwrap();
    let add_h = g.add_root_node(HFile::new("project_C/add.h")).unwrap();
    let wrapper_h = g.add_root_node(HFile::new("project_C/wrapper.h")).unwrap();
    let app = g.add_node(XFile::new("project_C/app")).unwrap();

    // Add explicit edges
    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, app);

    // Simulate scanned edges (header dependencies)
    g.add_edge(add_h, add_o); // add.c includes add.h
    g.add_edge(wrapper_h, main_o); // main.c includes wrapper.h
    g.add_edge(add_h, main_o); // wrapper.h includes add.h

    // Get root predecessors of main.o
    let roots = g.root_predecessors(main_o);
    let root_paths: Vec<PathBuf> = roots.iter().map(|&idx| g.g[idx].pathbuf()).collect();

    // main.c should be in the root predecessors
    assert!(
        root_paths.contains(&PathBuf::from("project_C/main.c")),
        "main.c should be a root predecessor of main.o"
    );

    // wrapper.h should be in the root predecessors
    assert!(
        root_paths.contains(&PathBuf::from("project_C/wrapper.h")),
        "wrapper.h should be a root predecessor of main.o"
    );

    // add.h should be in the root predecessors (via wrapper.h)
    assert!(
        root_paths.contains(&PathBuf::from("project_C/add.h")),
        "add.h should be a root predecessor of main.o"
    );

    // add.c should NOT be in the root predecessors of main.o
    assert!(
        !root_paths.contains(&PathBuf::from("project_C/add.c")),
        "add.c should NOT be a root predecessor of main.o"
    );

    // Verify count: main.c, wrapper.h, add.h = 3 roots
    assert_eq!(
        roots.len(),
        3,
        "main.o should have exactly 3 root predecessors"
    );
}
