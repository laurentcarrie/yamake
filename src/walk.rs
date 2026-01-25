use log::error;
use petgraph::Direction;
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::model::{G, GNodeStatus};
use crate::mount::mount;

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

    fn mount_root_nodes(&mut self) -> bool {
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
                if let Err(e) = mount(&self.srcdir, &self.sandbox, &node.pathbuf()) {
                    error!("Failed to mount {}: {}", node.id(), e);
                    self.nodes_status
                        .insert(node_idx, GNodeStatus::MountedFailed);
                    return false;
                }
                self.nodes_status.insert(node_idx, GNodeStatus::Mounted);
            }
        }
        true
    }

    pub fn make(&mut self) -> bool {
        self.print_status();

        // Mount root nodes
        if !self.mount_root_nodes() {
            return false;
        }
        self.print_status();

        // Add edges discovered by scanning
        self.add_scanned_edges();

        // Mount any new root nodes that became reachable after scanning
        if !self.mount_root_nodes() {
            return false;
        }
        self.print_status();

        let all_nodes: Vec<NodeIndex> = self.g.node_indices().collect();

        let mut built: HashSet<NodeIndex> = HashSet::new();
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
            let build_results: Mutex<HashMap<NodeIndex, bool>> = Mutex::new(HashMap::new());
            ready.par_iter().for_each(|&node_idx| {
                let predecessors: Vec<&Box<dyn crate::model::GNode + Send + Sync>> = self
                    .g
                    .neighbors_directed(node_idx, Direction::Incoming)
                    .map(|idx| &self.g[idx])
                    .collect();
                let node = &self.g[node_idx];
                let result = node.build(&self.sandbox, &predecessors);

                // Check that output file exists after successful build
                let result = if result {
                    let output_path = self.sandbox.join(node.pathbuf());
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

                build_results.lock().unwrap().insert(node_idx, result);
            });

            // Update status based on build results
            let results = build_results.into_inner().unwrap();
            for idx in ready {
                if *results.get(&idx).unwrap_or(&false) {
                    self.nodes_status.insert(idx, GNodeStatus::Build);
                } else {
                    self.nodes_status.insert(idx, GNodeStatus::BuildFailed);
                    // Mark all dependents as AncestorFailed
                    self.mark_dependents_failed(idx, &mut built);
                    success = false;
                }
                built.insert(idx);
            }
        }
        self.print_status();
        success
    }
}
