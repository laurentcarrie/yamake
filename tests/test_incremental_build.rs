//! Test incremental build behavior.

use std::path::PathBuf;
use tempdir::TempDir;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::{G, GNodeStatus};

/// Tests that a second build correctly identifies unchanged files.
///
/// After two consecutive builds with no source changes:
/// - Root nodes (source files) should be MountedNotChanged
/// - All other nodes (built artifacts) should be BuildNotRequired
#[test]
fn test_incremental_build_unchanged() {
    let srcdir = PathBuf::from("demo_projects");
    let sandbox = TempDir::new("yamake_test_incremental").unwrap();
    let sandbox_path = sandbox.path().to_path_buf();

    let mut g = G::new(srcdir, sandbox_path);

    let main_c = g.add_root_node(CFile::new("project_1/main.c")).unwrap();
    let main_o = g.add_node(OFile::new("project_1/main.o", vec![], vec![])).unwrap();
    let add_c = g.add_root_node(CFile::new("project_1/add.c")).unwrap();
    let add_o = g.add_node(OFile::new("project_1/add.o", vec![], vec!["-DYYY_defined".to_string()])).unwrap();
    let add_h = g.add_root_node(HFile::new("project_1/add.h")).unwrap();
    let wrapper_h = g.add_root_node(HFile::new("project_1/wrapper.h")).unwrap();
    let project_a = g.add_node(AFile::new("project_1/libproject.a")).unwrap();
    let app = g.add_node(XFile::new("project_1/app")).unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(add_c, add_o);
    g.add_edge(add_o, project_a);
    g.add_edge(project_a, app);

    // First build
    let result = g.make();
    assert!(result, "First build should succeed");

    // Second build - should detect no changes
    let result = g.make();
    assert!(result, "Second build should succeed");

    // Check root nodes are MountedNotChanged
    let root_nodes = vec![main_c, add_c, add_h, wrapper_h];
    for idx in root_nodes {
        let status = g.nodes_status.get(&idx);
        assert_eq!(
            status,
            Some(&GNodeStatus::MountedNotChanged),
            "Root node {} should be MountedNotChanged, got {:?}",
            g.g[idx].id(),
            status
        );
    }

    // Check non-root nodes are BuildNotRequired
    let built_nodes = vec![main_o, add_o, project_a, app];
    for idx in built_nodes {
        let status = g.nodes_status.get(&idx);
        assert_eq!(
            status,
            Some(&GNodeStatus::BuildNotRequired),
            "Built node {} should be BuildNotRequired, got {:?}",
            g.g[idx].id(),
            status
        );
    }
}
