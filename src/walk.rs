use log::{error, info};
use petgraph::Direction;
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Mutex;

use crate::model::{G, GNodeStatus, OutputInfo, PredecessorInfo};
use crate::mount::mount;

fn compute_file_digest(path: &std::path::Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer).ok()?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Some(hex::encode(hasher.finalize()))
}

fn load_digests(path: &std::path::Path) -> BTreeMap<String, String> {
    if let Ok(file) = File::open(path) {
        let infos: Vec<OutputInfo> = serde_yaml::from_reader(file).unwrap_or_default();
        infos
            .into_iter()
            .filter_map(|info| {
                info.digest
                    .map(|d| (info.pathbuf.to_string_lossy().to_string(), d))
            })
            .collect()
    } else {
        BTreeMap::new()
    }
}

impl G {
    fn mark_dependents_failed(&mut self, failed_idx: NodeIndex, built: &mut HashSet<NodeIndex>) {
        // Mark all nodes that depend on the failed node as AncestorFailed
        let mut to_mark: Vec<NodeIndex> = self
            .g
            .neighbors_directed(failed_idx, Direction::Outgoing)
            .collect();

        while let Some(idx) = to_mark.pop() {
            if self.nodes_status.get(&idx) == Some(&GNodeStatus::AncestorFailed) {
                continue;
            }
            self.nodes_status.insert(idx, GNodeStatus::AncestorFailed);
            built.insert(idx);

            // Add dependents of this node to the list
            for dependent in self.g.neighbors_directed(idx, Direction::Outgoing) {
                to_mark.push(dependent);
            }
        }
    }

    fn mount_root_nodes(&mut self, previous_digests: &BTreeMap<String, String>) -> bool {
        let node_indices: Vec<NodeIndex> = self.g.node_indices().collect();
        for node_idx in node_indices {
            if self.nodes_status.get(&node_idx) != Some(&GNodeStatus::Initial) {
                continue;
            }
            let has_predecessors = self
                .g
                .neighbors_directed(node_idx, Direction::Incoming)
                .next()
                .is_some();

            if !has_predecessors {
                let node = &self.g[node_idx];
                let pathbuf_str = node.pathbuf().to_string_lossy().to_string();

                // Compute digest of source file before mounting
                let source_path = self.srcdir.join(node.pathbuf());
                let current_digest = compute_file_digest(&source_path);

                // Check if digest changed
                let changed = match (&current_digest, previous_digests.get(&pathbuf_str)) {
                    (Some(current), Some(previous)) => current != previous,
                    _ => true, // If no previous digest or can't compute, consider changed
                };

                if let Err(e) = mount(&self.srcdir, &self.sandbox, &node.pathbuf()) {
                    error!("Failed to mount {}: {}", node.pathbuf().display(), e);
                    self.nodes_status
                        .insert(node_idx, GNodeStatus::MountedFailed);
                    return false;
                }

                if changed {
                    self.nodes_status
                        .insert(node_idx, GNodeStatus::MountedChanged);
                } else {
                    self.nodes_status
                        .insert(node_idx, GNodeStatus::MountedNotChanged);
                }
            }
        }
        true
    }

    pub fn make(&mut self) -> bool {
        // Reset all node statuses to Initial
        for node_idx in self.g.node_indices() {
            self.nodes_status.insert(node_idx, GNodeStatus::Initial);
        }
        self.print_status();

        // Load previous digests
        let digest_path = self.sandbox.join("make-output.yml");
        let previous_digests = load_digests(&digest_path);

        // Mount root nodes
        if !self.mount_root_nodes(&previous_digests) {
            return false;
        }
        self.print_status();

        // Add edges discovered by scanning
        self.add_scanned_edges();

        // Mount any new root nodes that became reachable after scanning
        if !self.mount_root_nodes(&previous_digests) {
            return false;
        }
        self.print_status();

        let all_nodes: Vec<NodeIndex> = self.g.node_indices().collect();

        // Initialize built set with already-mounted root nodes
        let mut built: HashSet<NodeIndex> = all_nodes
            .iter()
            .filter(|&idx| {
                matches!(
                    self.nodes_status.get(idx),
                    Some(GNodeStatus::MountedChanged) | Some(GNodeStatus::MountedNotChanged)
                )
            })
            .copied()
            .collect();
        let mut success = true;

        while built.len() < all_nodes.len() {
            // Find nodes ready to build (all predecessors are built)
            let ready: Vec<NodeIndex> = all_nodes
                .iter()
                .filter(|&idx| !built.contains(idx))
                .filter(|&idx| {
                    self.g
                        .neighbors_directed(*idx, Direction::Incoming)
                        .all(|pred| built.contains(&pred))
                })
                .copied()
                .collect();

            if ready.is_empty() {
                break;
            }

            // Set status to Running before building
            for &idx in &ready {
                self.nodes_status.insert(idx, GNodeStatus::Running);
            }
            self.print_status();

            // Build ready nodes in parallel and collect results
            let build_results: Mutex<HashMap<NodeIndex, GNodeStatus>> = Mutex::new(HashMap::new());
            ready.par_iter().for_each(|&node_idx| {
                // Assert all predecessors are in expected states
                let pred_indices: Vec<NodeIndex> = self
                    .g
                    .neighbors_directed(node_idx, Direction::Incoming)
                    .collect();

                for &pred_idx in &pred_indices {
                    let pred_status = self.nodes_status.get(&pred_idx);
                    assert!(
                        matches!(
                            pred_status,
                            Some(GNodeStatus::MountedChanged)
                                | Some(GNodeStatus::MountedNotChanged)
                                | Some(GNodeStatus::BuildSuccess)
                                | Some(GNodeStatus::BuildNotRequired)
                                | Some(GNodeStatus::BuildFailed)
                                | Some(GNodeStatus::AncestorFailed)
                        ),
                        "Predecessor {} has unexpected status {:?}",
                        self.g[pred_idx].pathbuf().display(),
                        pred_status
                    );
                }

                let node = &self.g[node_idx];
                let output_path = self.sandbox.join(node.pathbuf());
                let pathbuf_str = node.pathbuf().to_string_lossy().to_string();

                // Check if any predecessor failed
                let has_failed_predecessor = pred_indices.iter().any(|&pred_idx| {
                    matches!(
                        self.nodes_status.get(&pred_idx),
                        Some(GNodeStatus::BuildFailed) | Some(GNodeStatus::AncestorFailed)
                    )
                });

                if has_failed_predecessor {
                    // Delete pathbuf if it exists and set AncestorFailed
                    if output_path.exists() {
                        let _ = std::fs::remove_file(&output_path);
                    }
                    build_results
                        .lock()
                        .unwrap()
                        .insert(node_idx, GNodeStatus::AncestorFailed);
                    return;
                }

                // Check if all predecessors are unchanged (BuildNotRequired or MountedNotChanged)
                let all_predecessors_not_required = !pred_indices.is_empty()
                    && pred_indices.iter().all(|&pred_idx| {
                        matches!(
                            self.nodes_status.get(&pred_idx),
                            Some(GNodeStatus::BuildNotRequired)
                                | Some(GNodeStatus::MountedNotChanged)
                        )
                    });

                // Determine if we need to build
                let need_build = if all_predecessors_not_required {
                    // Check if output exists and digest matches
                    if output_path.exists() {
                        let current_digest = compute_file_digest(&output_path);
                        let previous_digest = previous_digests.get(&pathbuf_str);
                        match (&current_digest, previous_digest) {
                            (Some(current), Some(previous)) => current != previous,
                            _ => true, // No digest available, need to build
                        }
                    } else {
                        true // Output doesn't exist, need to build
                    }
                } else {
                    true // Some predecessor changed, need to build
                };

                if !need_build {
                    // Skip build, output is up-to-date
                    build_results
                        .lock()
                        .unwrap()
                        .insert(node_idx, GNodeStatus::BuildNotRequired);
                    return;
                }

                // Perform the build
                let predecessors: Vec<&(dyn crate::model::GNode + Send + Sync)> =
                    pred_indices.iter().map(|&idx| self.g[idx].as_ref()).collect();
                let build_ok = node.build(&self.sandbox, &predecessors);

                // Check that output file exists after successful build
                let build_ok = if build_ok {
                    if output_path.exists() {
                        true
                    } else {
                        error!(
                            "Build succeeded but output file not found: {}",
                            output_path.display()
                        );
                        false
                    }
                } else {
                    false
                };

                let final_status = if !build_ok {
                    GNodeStatus::BuildFailed
                } else {
                    // Check digest after build
                    let new_digest = compute_file_digest(&output_path);
                    let previous_digest = previous_digests.get(&pathbuf_str);
                    match (&new_digest, previous_digest) {
                        (Some(new), Some(prev)) if new == prev => GNodeStatus::BuildNotRequired,
                        _ => GNodeStatus::BuildSuccess,
                    }
                };

                build_results.lock().unwrap().insert(node_idx, final_status);
            });

            // Update status based on build results
            let results = build_results.into_inner().unwrap();
            for idx in ready {
                let status = results
                    .get(&idx)
                    .copied()
                    .unwrap_or(GNodeStatus::BuildFailed);
                self.nodes_status.insert(idx, status);

                if status == GNodeStatus::BuildFailed || status == GNodeStatus::AncestorFailed {
                    if status == GNodeStatus::BuildFailed {
                        // Mark all dependents as AncestorFailed
                        self.mark_dependents_failed(idx, &mut built);
                    }
                    success = false;
                }
                built.insert(idx);
            }
        }
        self.print_status();

        // Compute and save digests for all files
        self.save_digests();

        success
    }

    fn save_digests(&self) {
        let mut infos: Vec<OutputInfo> = Vec::new();

        for node_idx in self.g.node_indices() {
            let node = &self.g[node_idx];
            let file_path = self.sandbox.join(node.pathbuf());
            let status = self
                .nodes_status
                .get(&node_idx)
                .copied()
                .unwrap_or(GNodeStatus::Initial);
            let digest = compute_file_digest(&file_path);

            // Compute absolute paths
            let absolute_path = file_path.canonicalize().ok();
            let log_base = self.sandbox.join("logs").join(node.pathbuf());
            let stdout_file = log_base.with_file_name(format!(
                "{}.stdout",
                log_base.file_name().unwrap_or_default().to_string_lossy()
            ));
            let stderr_file = log_base.with_file_name(format!(
                "{}.stderr",
                log_base.file_name().unwrap_or_default().to_string_lossy()
            ));
            let stdout_path = stdout_file.canonicalize().ok();
            let stderr_path = stderr_file.canonicalize().ok();

            // Collect predecessors with their status
            let predecessors: Vec<PredecessorInfo> = self
                .g
                .neighbors_directed(node_idx, Direction::Incoming)
                .map(|pred_idx| {
                    let pred_node = &self.g[pred_idx];
                    let pred_status = self
                        .nodes_status
                        .get(&pred_idx)
                        .copied()
                        .unwrap_or(GNodeStatus::Initial);
                    PredecessorInfo {
                        pathbuf: pred_node.pathbuf(),
                        status: pred_status,
                    }
                })
                .collect();

            infos.push(OutputInfo {
                pathbuf: node.pathbuf(),
                status,
                digest,
                absolute_path,
                stdout_path,
                stderr_path,
                predecessors,
            });
        }

        // Sort by pathbuf for consistent output
        infos.sort_by(|a, b| a.pathbuf.cmp(&b.pathbuf));

        let digest_path = self.sandbox.join("make-output.yml");
        match File::create(&digest_path) {
            Ok(file) => {
                if let Err(e) = serde_yaml::to_writer(file, &infos) {
                    error!("Failed to write make-output.yml: {e}");
                } else {
                    info!("Saved {} entries to {}", infos.len(), digest_path.display());
                }
            }
            Err(e) => {
                error!("Failed to create make-output.yml: {e}");
            }
        }
    }
}
