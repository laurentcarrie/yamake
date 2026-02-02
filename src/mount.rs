use crate::make::compute_file_digest;
use crate::model::{G, GNodeStatus};
use log::error;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn mount(srcdir: &Path, sandbox: &Path, p: &Path) -> io::Result<()> {
    let src_path = srcdir.join(p);
    let dest_path = sandbox.join(p);

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&src_path, &dest_path)?;
    Ok(())
}

impl G {
    /// Mount all root nodes that have Initial status.
    ///
    /// Root nodes are nodes with no predecessors (incoming edges).
    /// Uses previous_digests to determine if the file has changed.
    pub(crate) fn mount_root_nodes(&mut self, previous_digests: &HashMap<String, String>) {
        let node_indices: Vec<_> = self.g.node_indices().collect();

        for node_idx in node_indices {
            // Only process nodes with Initial status
            if self.nodes_status.get(&node_idx) != Some(&GNodeStatus::Initial) {
                continue;
            }

            // Only mount root nodes (nodes with no predecessors)
            let has_predecessors = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .next()
                .is_some();
            if has_predecessors {
                continue;
            }

            let pathbuf = self.g[node_idx].pathbuf();
            let pathbuf_str = pathbuf.to_string_lossy().to_string();

            // Compute digest of source file before mounting
            let source_path = self.srcdir.join(&pathbuf);
            let current_digest = compute_file_digest(&source_path);

            // Mount the file from srcdir to sandbox
            if let Err(e) = mount(&self.srcdir, &self.sandbox, &pathbuf) {
                error!("Failed to mount {}: {}", pathbuf.display(), e);
                self.nodes_status.insert(node_idx, GNodeStatus::MountedFailed);
            } else {
                // Compare current digest with previous to determine if changed
                let status = match (&current_digest, previous_digests.get(&pathbuf_str)) {
                    (Some(current), Some(previous)) if current == previous => {
                        GNodeStatus::MountedNotChanged
                    }
                    _ => GNodeStatus::MountedChanged,
                };
                self.nodes_status.insert(node_idx, status);
            }
        }
    }
}
