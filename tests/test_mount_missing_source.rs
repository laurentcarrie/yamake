//! Test that a missing source file causes the build to fail.
//!
//! When a CFile references a source that does not exist in the source directory,
//! mount should fail and make() should return false.

use tempdir::TempDir;
use yamake::c_nodes::{CFile, OFile};
use yamake::model::{G, GNodeStatus};

#[test]
fn test_mount_missing_source() {
    let srcdir = TempDir::new("yamake_test_srcdir").unwrap();
    let sandbox = TempDir::new("yamake_test_sandbox").unwrap();
    let srcdir_path = srcdir.path().to_path_buf();
    let sandbox_path = sandbox.path().to_path_buf();

    // Do NOT create any source file — "missing/hello.c" does not exist

    let mut g = G::new(srcdir_path, sandbox_path.clone());

    let hello_c = g.add_root_node(CFile::new("missing/hello.c")).unwrap();
    let hello_o = g
        .add_node(OFile::new("missing/hello.o", vec![], vec![]))
        .unwrap();
    g.add_edge(hello_c, hello_o);

    let result = g.make();
    assert!(
        !result,
        "make should fail when a source file cannot be mounted"
    );

    let c_status = g.nodes_status.get(&hello_c);
    assert_eq!(
        c_status,
        Some(&GNodeStatus::MountedFailed),
        "hello.c should be MountedFailed, got {c_status:?}"
    );

    let o_status = g.nodes_status.get(&hello_o);
    assert_eq!(
        o_status,
        Some(&GNodeStatus::AncestorFailed),
        "hello.o should be AncestorFailed, got {o_status:?}"
    );
}
