use log::{error, info};
use petgraph::Direction;
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Mutex;

use crate::model::{EdgeType, ExpandedEdgeInfo, G, GNodeStatus, MakeOutput, OutputInfo, PredecessorInfo};
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

fn load_make_output(path: &std::path::Path) -> Option<MakeOutput> {
    let file = File::open(path).ok()?;
    serde_yaml::from_reader(file).ok()
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
        // Load previous digests from make-output.yml
        let digest_path = self.sandbox.join("make-output.yml");
        let previous_digests: BTreeMap<String, String> = load_make_output(&digest_path)
            .map(|output| {
                output
                    .nodes
                    .into_iter()
                    .filter_map(|info| {
                        info.digest
                            .map(|d| (info.pathbuf.to_string_lossy().to_string(), d))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Track expanded nodes during this build
        let mut expanded_nodes: HashSet<NodeIndex> = HashSet::new();

        // Reset all node statuses to Initial
        for node_idx in self.g.node_indices() {
            self.nodes_status.insert(node_idx, GNodeStatus::Initial);
        }
        self.print_status();

        // Mount root nodes
        if !self.mount_root_nodes(&previous_digests) {
            return false;
        }
        self.print_status();

        let mut all_nodes: Vec<NodeIndex> = self.g.node_indices().collect();

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
        let mut progress_made = true; // Track if progress was made in the last iteration

        while built.len() < all_nodes.len() {
            // Find nodes potentially ready to build (all current predecessors are built)
            let potentially_ready: Vec<NodeIndex> = all_nodes
                .iter()
                .filter(|&idx| !built.contains(idx))
                .filter(|&idx| {
                    self.g
                        .neighbors_directed(*idx, Direction::Incoming)
                        .all(|pred| built.contains(&pred))
                })
                .copied()
                .collect();

            if potentially_ready.is_empty() {
                break;
            }

            // Only reset ScanIncomplete nodes if progress was made (to avoid infinite loops)
            if progress_made {
                for &node_idx in &potentially_ready {
                    if self.nodes_status.get(&node_idx) == Some(&GNodeStatus::ScanIncomplete) {
                        self.nodes_status.insert(node_idx, GNodeStatus::Running);
                    }
                }
            }
            progress_made = false; // Reset for this iteration

            // Scan each potentially ready node to discover additional dependencies
            for &node_idx in &potentially_ready {
                let predecessors: Vec<&(dyn crate::model::GNode + Send + Sync)> = self
                    .g
                    .neighbors_directed(node_idx, Direction::Incoming)
                    .map(|idx| self.g[idx].as_ref())
                    .collect();

                let (scan_complete, scanned_paths) = self.g[node_idx].scan(&self.sandbox, &predecessors);

                // Add edges for scanned dependencies
                let mut has_unbuilt_graph_dependency = false;
                let mut has_orphan_sandbox_file = false;
                for path in &scanned_paths {
                    let found_node = self
                        .g
                        .node_indices()
                        .find(|&idx| self.g[idx].pathbuf() == *path);

                    if let Some(from_idx) = found_node {
                        // Check if edge already exists
                        let edge_exists = self.g.edges_connecting(from_idx, node_idx).next().is_some();
                        if !edge_exists {
                            self.g.add_edge(from_idx, node_idx, crate::model::EdgeType::Scanned);
                        }
                        // Check if this dependency is not yet built
                        if !built.contains(&from_idx) {
                            has_unbuilt_graph_dependency = true;
                        }
                    } else {
                        // File exists in sandbox but has no graph node - likely from previous expand
                        // We should wait for expand to run and potentially update this file
                        let file_path = self.sandbox.join(path);
                        if file_path.exists() {
                            has_orphan_sandbox_file = true;
                        }
                    }
                }

                // Mark as ScanIncomplete if:
                // 1. There's a dependency in the graph that's not built yet, OR
                // 2. The scan is incomplete (missing files) AND there are other unbuilt nodes
                //    that could potentially generate the missing file via expand, OR
                // 3. There's a scanned file in sandbox with no graph node (from previous expand)
                //    AND there are other unbuilt nodes that might update it
                let has_other_unbuilt_nodes = all_nodes.iter().any(|&idx| !built.contains(&idx) && idx != node_idx);
                if has_unbuilt_graph_dependency || ((!scan_complete || has_orphan_sandbox_file) && has_other_unbuilt_nodes) {
                    self.nodes_status.insert(node_idx, GNodeStatus::ScanIncomplete);
                }
            }

            // Mount any new root nodes that became reachable after scanning
            if !self.mount_root_nodes(&previous_digests) {
                return false;
            }

            // Re-evaluate which nodes are truly ready (after adding scanned edges)
            // Exclude nodes with incomplete scans
            let ready: Vec<NodeIndex> = potentially_ready
                .iter()
                .filter(|&idx| {
                    self.nodes_status.get(idx) != Some(&GNodeStatus::ScanIncomplete)
                })
                .filter(|&idx| {
                    self.g
                        .neighbors_directed(*idx, Direction::Incoming)
                        .all(|pred| built.contains(&pred))
                })
                .copied()
                .collect();

            if ready.is_empty() {
                if !progress_made {
                    // No progress was made and no nodes are ready - we're stuck
                    info!("No ready nodes and no progress made - breaking out of build loop");
                    break;
                }
                // Scanned edges revealed new dependencies, continue loop to process them
                continue;
            }

            // Set status to Running before building
            for &idx in &ready {
                self.nodes_status.insert(idx, GNodeStatus::Running);
            }
            self.print_status();

            // Build ready nodes in parallel and collect results
            let build_results: Mutex<HashMap<NodeIndex, (GNodeStatus, crate::model::ExpandResult)>> = Mutex::new(HashMap::new());
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
                        .insert(node_idx, (GNodeStatus::AncestorFailed, (Vec::new(), Vec::new())));
                    return;
                }

                // Build predecessors list for expand and build calls
                let predecessors: Vec<&(dyn crate::model::GNode + Send + Sync)> =
                    pred_indices.iter().map(|&idx| self.g[idx].as_ref()).collect();

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
                    // Skip build, output is up-to-date, but still run expand
                    let expand_result = node.expand(&self.sandbox, &predecessors);
                    build_results
                        .lock()
                        .unwrap()
                        .insert(node_idx, (GNodeStatus::BuildNotRequired, expand_result));
                    return;
                }

                // Perform the build
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

                // Call expand after building
                let expand_result = node.expand(&self.sandbox, &predecessors);

                build_results.lock().unwrap().insert(node_idx, (final_status, expand_result));
            });

            // Update status based on build results
            let mut results = build_results.into_inner().unwrap();
            for idx in ready.clone() {
                let (status, (new_expanded_nodes, new_expanded_edges)) = results
                    .remove(&idx)
                    .unwrap_or((GNodeStatus::BuildFailed, (Vec::new(), Vec::new())));
                self.nodes_status.insert(idx, status);

                if status == GNodeStatus::BuildFailed || status == GNodeStatus::AncestorFailed {
                    if status == GNodeStatus::BuildFailed {
                        // Mark all dependents as AncestorFailed
                        self.mark_dependents_failed(idx, &mut built);
                    }
                    success = false;
                }
                built.insert(idx);
                progress_made = true; // Progress was made - a node was built

                // Add expanded nodes to the graph (skip if already exists)
                for exp_node in new_expanded_nodes {
                    let pathbuf = exp_node.pathbuf();
                    let existing = self.g.node_indices().find(|&i| self.g[i].pathbuf() == pathbuf);
                    if existing.is_none() {
                        let new_idx = self.g.add_node(exp_node);
                        self.nodes_status.insert(new_idx, GNodeStatus::MountedChanged);
                        expanded_nodes.insert(new_idx);
                        all_nodes.push(new_idx);
                        // If the file already exists (expand created it), mark as built
                        // so other nodes can proceed with their scanned dependencies
                        let file_path = self.sandbox.join(&pathbuf);
                        if file_path.exists() {
                            built.insert(new_idx);
                        }
                    }
                }

                // Add expanded edges between nodes (skip if already exists)
                for edge in new_expanded_edges {
                    let from_pathbuf = edge.nfrom.pathbuf();
                    let to_pathbuf = edge.nto.pathbuf();
                    let from_idx = self.g.node_indices().find(|&i| self.g[i].pathbuf() == from_pathbuf);
                    let to_idx = self.g.node_indices().find(|&i| self.g[i].pathbuf() == to_pathbuf);
                    if let (Some(from), Some(to)) = (from_idx, to_idx) {
                        // Check if edge already exists
                        let edge_exists = self.g.edges_connecting(from, to).next().is_some();
                        if !edge_exists {
                            self.g.add_edge(from, to, crate::model::EdgeType::Expanded);
                        }
                    }
                }
            }
        }
        self.print_status();

        // Compute and save digests for all files
        self.save_digests(&expanded_nodes);

        success
    }

    fn save_digests(&self, expanded_nodes: &HashSet<NodeIndex>) {
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

            let expanded = expanded_nodes.contains(&node_idx);
            let tag = if expanded {
                Some(node.tag())
            } else {
                None
            };

            infos.push(OutputInfo {
                pathbuf: node.pathbuf(),
                status,
                digest,
                absolute_path,
                stdout_path,
                stderr_path,
                predecessors,
                expanded,
                tag,
            });
        }

        // Sort by pathbuf for consistent output
        infos.sort_by(|a, b| a.pathbuf.cmp(&b.pathbuf));

        // Collect expanded edges
        let mut expanded_edges: Vec<ExpandedEdgeInfo> = Vec::new();
        for edge_idx in self.g.edge_indices() {
            if let Some(edge_type) = self.g.edge_weight(edge_idx) {
                if *edge_type == EdgeType::Expanded {
                    if let Some((from, to)) = self.g.edge_endpoints(edge_idx) {
                        expanded_edges.push(ExpandedEdgeInfo {
                            from_pathbuf: self.g[from].pathbuf(),
                            to_pathbuf: self.g[to].pathbuf(),
                            edge_type: EdgeType::Expanded,
                        });
                    }
                }
            }
        }

        let output = MakeOutput {
            nodes: infos,
            expanded_edges,
        };

        let digest_path = self.sandbox.join("make-output.yml");
        match File::create(&digest_path) {
            Ok(file) => {
                if let Err(e) = serde_yaml::to_writer(file, &output) {
                    error!("Failed to write make-output.yml: {e}");
                } else {
                    info!("Saved {} nodes, {} expanded edges to {}",
                        output.nodes.len(), output.expanded_edges.len(), digest_path.display());
                }
            }
            Err(e) => {
                error!("Failed to create make-output.yml: {e}");
            }
        }
    }
}
