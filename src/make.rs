use crate::model::{EdgeType, G, GNodeStatus, MakeOutput, OutputInfo, PredecessorInfo};
use log::error;
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;

pub(crate) fn compute_file_digest(path: &Path) -> Option<String> {
    let content = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Some(hex::encode(hasher.finalize()))
}

fn load_previous_digests(path: &Path) -> HashMap<String, String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };
    let output: MakeOutput = match serde_yaml::from_reader(file) {
        Ok(o) => o,
        Err(_) => return HashMap::new(),
    };
    output
        .nodes
        .into_iter()
        .filter_map(|info| {
            info.digest
                .map(|d| (info.pathbuf.to_string_lossy().to_string(), d))
        })
        .collect()
}

impl G {
    pub fn make(&mut self) -> bool {
        // Load previous digests from make-report.yml
        let report_path = self.sandbox.join("make-report.yml");
        let previous_digests = load_previous_digests(&report_path);

        // Set all node statuses to Initial
        for node_idx in self.g.node_indices() {
            self.nodes_status.insert(node_idx, GNodeStatus::Initial);
        }

        loop {
            let digest_before = self.graph_digest();

            // Mount root nodes
            self.mount_root_nodes(&previous_digests);

            // Reset ScanIncomplete nodes to Initial so they can be re-scanned
            for node_idx in self.g.node_indices() {
                if self.nodes_status.get(&node_idx) == Some(&GNodeStatus::ScanIncomplete) {
                    self.nodes_status.insert(node_idx, GNodeStatus::Initial);
                }
            }

            // Expand nodes first (may add new nodes and edges)
            self.expand_nodes();

            // Scan nodes to discover dependencies (after expand, so generated nodes exist)
            self.scan_nodes();

            // Build nodes
            self.build_nodes(&previous_digests);

            // Print status summary for this iteration
            self.print_status();

            // Verify all nodes have a status
            assert_eq!(
                self.nodes_status.len(),
                self.g.node_count(),
                "Status count ({}) does not match node count ({})",
                self.nodes_status.len(),
                self.g.node_count()
            );

            let digest_after = self.graph_digest();
            if digest_after == digest_before {
                break;
            }
        }

        // Save digests to make-report.yml
        self.save_digests();

        // Return false if any node failed
        !self.nodes_status.values().any(|&status| {
            matches!(
                status,
                GNodeStatus::BuildFailed | GNodeStatus::MountedFailed | GNodeStatus::AncestorFailed
            )
        })
    }

    /// Save digests and status to make-report.yml.
    fn save_digests(&self) {
        let mut infos: Vec<OutputInfo> = Vec::new();

        for node_idx in self.g.node_indices() {
            let node = &self.g[node_idx];
            let pathbuf = node.pathbuf();
            let file_path = self.sandbox.join(&pathbuf);
            let status = self
                .nodes_status
                .get(&node_idx)
                .copied()
                .unwrap_or(GNodeStatus::Initial);
            let digest = compute_file_digest(&file_path);

            // Collect predecessors with their status
            let predecessors: Vec<PredecessorInfo> = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
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
                pathbuf,
                status,
                digest,
                absolute_path: file_path.canonicalize().ok(),
                stdout_path: None,
                stderr_path: None,
                predecessors,
                expanded: false,
                tag: node.tag(),
            });
        }

        // Sort by pathbuf for consistent output
        infos.sort_by(|a, b| a.pathbuf.cmp(&b.pathbuf));

        let output = MakeOutput { nodes: infos };
        let report_path = self.sandbox.join("make-report.yml");

        match File::create(&report_path) {
            Ok(file) => {
                if let Err(e) = serde_yaml::to_writer(file, &output) {
                    error!("Failed to write make-report.yml: {e}");
                }
            }
            Err(e) => {
                error!("Failed to create make-report.yml: {e}");
            }
        }
    }

    /// Scan all nodes to discover dependencies.
    ///
    /// For each node, call its scan method to find additional dependencies
    /// and add edges for any discovered dependencies that exist in the graph.
    fn scan_nodes(&mut self) {
        let node_indices: Vec<NodeIndex> = self.g.node_indices().collect();

        for node_idx in node_indices {
            // Get predecessors for this node
            let pred_indices: Vec<NodeIndex> = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .collect();

            let predecessors: Vec<&(dyn crate::model::GNode + Send + Sync)> = pred_indices
                .iter()
                .map(|&idx| self.g[idx].as_ref())
                .collect();

            // Call scan on the node
            let (scan_complete, scanned_paths) =
                self.g[node_idx].scan(&self.sandbox, &predecessors);

            // If scan is not complete, mark node as ScanIncomplete
            if !scan_complete {
                self.nodes_status
                    .insert(node_idx, GNodeStatus::ScanIncomplete);
            }

            // Add edges for discovered dependencies
            for path in scanned_paths {
                // Find if there's a node with this path
                let found_node = self
                    .g
                    .node_indices()
                    .find(|&idx| self.g[idx].pathbuf() == path);

                if let Some(from_idx) = found_node {
                    // Check if edge already exists
                    let edge_exists = self.g.edges_connecting(from_idx, node_idx).next().is_some();
                    if !edge_exists {
                        self.g.add_edge(from_idx, node_idx, EdgeType::Scanned);
                        // If the dependency is new or has a "changed" status, reset the
                        // target node so it will be rebuilt with the new dependency
                        let from_status = self.nodes_status.get(&from_idx);
                        let needs_rebuild = matches!(
                            from_status,
                            Some(GNodeStatus::Initial)
                                | Some(GNodeStatus::MountedChanged)
                                | Some(GNodeStatus::BuildSuccess)
                                | Some(GNodeStatus::BuildNotChanged)
                        );
                        if needs_rebuild {
                            self.nodes_status.insert(node_idx, GNodeStatus::Initial);
                        }
                    }
                }
            }
        }
    }

    /// Build nodes that are ready.
    ///
    /// A node is ready to build when all its predecessors have been built or mounted.
    /// Nodes with ScanIncomplete status are skipped.
    /// If all predecessors are unchanged and output exists with same digest, skip build.
    /// Builds are executed concurrently using Rayon.
    fn build_nodes(&mut self, previous_digests: &HashMap<String, String>) {
        let node_indices: Vec<NodeIndex> = self.g.node_indices().collect();

        // First pass: mark AncestorFailed and BuildNotRequired nodes (no actual building)
        let mut nodes_to_expand: Vec<NodeIndex> = Vec::new();
        let mut nodes_to_build: Vec<NodeIndex> = Vec::new();

        for node_idx in node_indices {
            // Skip nodes that are not in Initial or ScanIncomplete status
            if !matches!(
                self.nodes_status.get(&node_idx),
                Some(&GNodeStatus::Initial) | Some(&GNodeStatus::ScanIncomplete)
            ) {
                continue;
            }

            // Skip root nodes (they don't need building - nodes with no predecessors)
            let has_predecessors = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .next()
                .is_some();
            if !has_predecessors {
                continue;
            }

            // Check if any predecessor failed (check this before Initial wait)
            let has_failed_predecessor = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .any(|pred_idx| {
                    matches!(
                        self.nodes_status.get(&pred_idx),
                        Some(GNodeStatus::BuildFailed)
                            | Some(GNodeStatus::AncestorFailed)
                            | Some(GNodeStatus::MountedFailed)
                    )
                });

            if has_failed_predecessor {
                self.nodes_status
                    .insert(node_idx, GNodeStatus::AncestorFailed);
                continue;
            }

            // Only Initial nodes proceed to build; ScanIncomplete without failed
            // predecessors will be retried after more progress is made
            if self.nodes_status.get(&node_idx) != Some(&GNodeStatus::Initial) {
                continue;
            }

            // Check if any predecessor is still Initial - wait for it
            let has_initial_predecessor = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .any(|pred_idx| self.nodes_status.get(&pred_idx) == Some(&GNodeStatus::Initial));

            if has_initial_predecessor {
                continue;
            }

            // Check if all predecessors are ready (built or mounted)
            let all_predecessors_ready = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .all(|pred_idx| {
                    matches!(
                        self.nodes_status.get(&pred_idx),
                        Some(GNodeStatus::MountedChanged)
                            | Some(GNodeStatus::MountedNotChanged)
                            | Some(GNodeStatus::BuildSuccess)
                            | Some(GNodeStatus::BuildNotChanged)
                            | Some(GNodeStatus::BuildNotRequired)
                    )
                });

            if !all_predecessors_ready {
                continue;
            }

            // Check if all predecessors are unchanged
            let all_predecessors_unchanged = self
                .g
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .all(|pred_idx| {
                    matches!(
                        self.nodes_status.get(&pred_idx),
                        Some(GNodeStatus::MountedNotChanged)
                            | Some(GNodeStatus::BuildNotChanged)
                            | Some(GNodeStatus::BuildNotRequired)
                    )
                });

            // If all predecessors unchanged, check if output exists with same digest
            if all_predecessors_unchanged {
                let pathbuf = self.g[node_idx].pathbuf();
                let pathbuf_str = pathbuf.to_string_lossy().to_string();
                let output_path = self.sandbox.join(&pathbuf);

                if output_path.exists() {
                    let current_digest = compute_file_digest(&output_path);
                    if let (Some(current), Some(previous)) =
                        (&current_digest, previous_digests.get(&pathbuf_str))
                    {
                        if current == previous {
                            self.nodes_status
                                .insert(node_idx, GNodeStatus::BuildNotRequired);
                            nodes_to_expand.push(node_idx);
                            continue;
                        }
                    }
                }
            }

            // Node is ready to build
            nodes_to_build.push(node_idx);
        }

        // Expand nodes marked as BuildNotRequired
        for node_idx in nodes_to_expand {
            self.expand_single_node(node_idx);
        }

        // Build nodes concurrently
        let build_results: Vec<(NodeIndex, GNodeStatus)> = nodes_to_build
            .par_iter()
            .map(|&node_idx| {
                // Get predecessors for the build call
                let pred_indices: Vec<NodeIndex> = self
                    .g
                    .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                    .collect();

                let predecessors: Vec<&(dyn crate::model::GNode + Send + Sync)> = pred_indices
                    .iter()
                    .map(|&idx| self.g[idx].as_ref())
                    .collect();

                // Build the node
                let build_ok = self.g[node_idx].build(&self.sandbox, &predecessors);

                // Determine status based on build result
                let status = if build_ok {
                    let pathbuf = self.g[node_idx].pathbuf();
                    let pathbuf_str = pathbuf.to_string_lossy().to_string();
                    let output_path = self.sandbox.join(&pathbuf);

                    if !output_path.exists() {
                        error!(
                            "build succeeded but output file missing: {}",
                            output_path.display()
                        );
                        GNodeStatus::BuildFailed
                    } else {
                        let current_digest = compute_file_digest(&output_path);

                        match (&current_digest, previous_digests.get(&pathbuf_str)) {
                            (Some(current), Some(previous)) if current == previous => {
                                GNodeStatus::BuildNotChanged
                            }
                            _ => GNodeStatus::BuildSuccess,
                        }
                    }
                } else {
                    GNodeStatus::BuildFailed
                };

                (node_idx, status)
            })
            .collect();

        // Update statuses from build results
        for (node_idx, status) in build_results {
            self.nodes_status.insert(node_idx, status);
        }
    }

    /// Expand a single node, adding any new nodes and edges to the graph.
    fn expand_single_node(&mut self, node_idx: NodeIndex) {
        // Get predecessor indices
        let pred_indices: Vec<NodeIndex> = self
            .g
            .neighbors_directed(node_idx, petgraph::Direction::Incoming)
            .collect();

        // Clone sandbox path to avoid borrow issues
        let sandbox = self.sandbox.clone();

        // Build predecessors vector
        let predecessors: Vec<&(dyn crate::model::GNode + Send + Sync)> = pred_indices
            .iter()
            .map(|&idx| self.g[idx].as_ref())
            .collect();

        // Call expand on the node
        let expand_result = self.g[node_idx].expand(&sandbox, &predecessors);

        // Handle expand result - add new nodes and edges
        if let Ok((new_nodes, new_edges)) = expand_result {
            for node in new_nodes {
                let pathbuf = node.pathbuf();
                // Check if node already exists
                let exists = self
                    .g
                    .node_indices()
                    .any(|idx| self.g[idx].pathbuf() == pathbuf);
                if !exists {
                    let new_idx = self.g.add_node(node);
                    self.nodes_status.insert(new_idx, GNodeStatus::Initial);
                }
            }

            for edge in new_edges {
                let from_path = edge.nfrom.pathbuf();
                let to_path = edge.nto.pathbuf();
                let from_idx = self
                    .g
                    .node_indices()
                    .find(|&idx| self.g[idx].pathbuf() == from_path);
                let to_idx = self
                    .g
                    .node_indices()
                    .find(|&idx| self.g[idx].pathbuf() == to_path);
                if let (Some(from), Some(to)) = (from_idx, to_idx) {
                    let edge_exists = self.g.edges_connecting(from, to).next().is_some();
                    if !edge_exists {
                        self.g.add_edge(from, to, EdgeType::Expanded);
                        // Reset target node to Initial if it failed (may have been
                        // built without all predecessors)
                        if self.nodes_status.get(&to) == Some(&GNodeStatus::BuildFailed) {
                            self.nodes_status.insert(to, GNodeStatus::Initial);
                        }
                    }
                }
            }
        }
    }

    /// Expand nodes that have been built or mounted.
    ///
    /// Calls expand on each node that is ready, which may add new nodes and edges to the graph.
    fn expand_nodes(&mut self) {
        let node_indices: Vec<NodeIndex> = self.g.node_indices().collect();

        for node_idx in node_indices {
            // Only expand nodes that have been built or mounted
            let status = self.nodes_status.get(&node_idx);
            if !matches!(
                status,
                Some(GNodeStatus::MountedChanged)
                    | Some(GNodeStatus::MountedNotChanged)
                    | Some(GNodeStatus::BuildSuccess)
                    | Some(GNodeStatus::BuildNotChanged)
                    | Some(GNodeStatus::BuildNotRequired)
            ) {
                continue;
            }

            self.expand_single_node(node_idx);
        }
    }

    /// Computes a digest of the entire graph state.
    ///
    /// For each node in alphabetical order by pathbuf:
    /// - If the file exists in the sandbox, compute its digest
    /// - Otherwise use "none"
    ///
    /// Also includes node statuses to detect when work remains to be done.
    /// Concatenate all digests and return the digest of that string.
    pub fn graph_digest(&self) -> String {
        // Collect all (pathbuf, status) pairs and sort by pathbuf
        let mut items: Vec<_> = self
            .g
            .node_indices()
            .map(|idx| {
                let pathbuf = self.g[idx].pathbuf();
                let status = self
                    .nodes_status
                    .get(&idx)
                    .copied()
                    .unwrap_or(GNodeStatus::Initial);
                (pathbuf, status)
            })
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));

        // Compute digest for each path, including status
        let mut combined = String::new();
        for (path, status) in items {
            let full_path = self.sandbox.join(&path);
            let file_digest = compute_file_digest(&full_path).unwrap_or_else(|| "none".to_string());
            combined.push_str(&file_digest);
            combined.push_str(&format!("{status:?}"));
        }

        // Return digest of combined string
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        hex::encode(hasher.finalize())
    }
}
