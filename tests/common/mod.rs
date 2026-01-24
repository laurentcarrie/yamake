//! Common test utilities and mock node types.
//!
//! This module provides simple implementations of `GNode` and `GRootNode` traits
//! for use in tests that don't require actual file compilation.

use std::path::PathBuf;
use yamake::model::{GNode, GRootNode};

/// A mock source file node for testing graph construction.
///
/// Implements `GRootNode` to represent a source file that exists in the
/// source directory and doesn't need to be built.
pub struct SourceFile {
    pub name: String,
}

impl GRootNode for SourceFile {
    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "SourceFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}

/// A mock target file node for testing graph construction.
///
/// Implements `GNode` to represent a build target. The `build()` method
/// always succeeds without producing any actual output.
pub struct TargetFile {
    pub name: String,
    pub path: PathBuf,
}

impl GNode for TargetFile {
    fn build(
        &self,
        _sandbox: &PathBuf,
        predecessors: &[&Box<dyn GNode + Send + Sync>],
    ) -> bool {
        let _inputs: Vec<String> = predecessors.iter().map(|p| p.id()).collect();
        true
    }

    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "TargetFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        self.path.clone()
    }
}
