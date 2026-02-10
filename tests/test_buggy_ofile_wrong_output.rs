//! Test that a buggy OFile writing to the wrong -o path fails to produce the expected output.
//!
//! The BuggyOFile passes "foobar" to gcc's -o option instead of the correct pathbuf,
//! so the compiled object file ends up at the wrong location and the expected output
//! is never built.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;
use yamake::c_nodes::CFile;
use yamake::command::run_command;
use yamake::model::{G, GNode};

/// A buggy OFile that passes "foobar" to -o instead of the correct pathbuf.
struct BuggyOFile {
    name: String,
}

impl BuggyOFile {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GNode for BuggyOFile {
    fn build(&self, sandbox: &Path, predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        let inputs: Vec<PathBuf> = predecessors
            .iter()
            .filter(|p| p.tag() == "CFile")
            .map(|p| sandbox.join(p.pathbuf()))
            .collect();

        let mut cmd = Command::new("gcc");
        cmd.arg("-c");
        cmd.arg("-I").arg(sandbox);
        // BUG: -o uses "foobar" instead of the correct pathbuf
        cmd.arg("-o").arg(sandbox.join("foobar"));
        for input in &inputs {
            cmd.arg(input);
        }

        run_command(&mut cmd, sandbox, &self.name)
    }

    fn tag(&self) -> String {
        "OFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}

/// This test fails because the BuggyOFile writes the compiled object to "foobar"
/// instead of "simple/hello.o", so the expected output file is never produced.
#[test]
fn test_buggy_ofile_wrong_output() {
    let srcdir = TempDir::new("yamake_test_srcdir").unwrap();
    let sandbox = TempDir::new("yamake_test_sandbox").unwrap();
    let srcdir_path = srcdir.path().to_path_buf();
    let sandbox_path = sandbox.path().to_path_buf();

    // Create a simple C file that compiles without any includes
    let c_dir = srcdir_path.join("simple");
    fs::create_dir_all(&c_dir).unwrap();
    fs::write(c_dir.join("hello.c"), "int main() { return 0; }\n").unwrap();

    let mut g = G::new(srcdir_path, sandbox_path.clone());

    let hello_c = g.add_root_node(CFile::new("simple/hello.c")).unwrap();
    let hello_o = g.add_node(BuggyOFile::new("simple/hello.o")).unwrap();
    g.add_edge(hello_c, hello_o);

    let result = g.make();
    assert!(
        !result,
        "make should fail because the output file is not at the expected path"
    );

    // The .o file was written to "foobar", not "simple/hello.o"
    let expected_o = sandbox_path.join("simple/hello.o");
    assert!(
        !expected_o.exists(),
        "Object file should NOT exist at {expected_o:?}"
    );

    let wrong_o = sandbox_path.join("foobar");
    assert!(
        wrong_o.exists(),
        "Object file was written to the wrong path {wrong_o:?}"
    );
}
