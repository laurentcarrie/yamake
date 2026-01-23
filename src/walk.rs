use std::collections::HashSet;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, Reversed};
use petgraph::Direction;
use rayon::prelude::*;

use crate::model::{G, mount};

impl G {
    pub fn walk_graph(&self, start: NodeIndex) {
        // Walk backwards through the graph (follow incoming edges)
        let reversed = Reversed(&self.g);
        let mut dfs = Dfs::new(&reversed, start);
        while let Some(node_idx) = dfs.next(&reversed) {
            let node = &self.g[node_idx];
            println!("{}", node.id());
        }
    }

    pub fn build_graph(&self, start: NodeIndex) {
        // Collect all reachable nodes (traversing backwards)
        let reversed = Reversed(&self.g);
        let mut all_nodes: Vec<NodeIndex> = Vec::new();
        let mut dfs = Dfs::new(&reversed, start);
        while let Some(node_idx) = dfs.next(&reversed) {
            all_nodes.push(node_idx);
        }

        let mut built: HashSet<NodeIndex> = HashSet::new();

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

            // Build ready nodes in parallel
            ready.par_iter().for_each(|&node_idx| {
                let predecessors: Vec<&Box<dyn crate::model::GNode + Send + Sync>> = self
                    .g
                    .neighbors_directed(node_idx, Direction::Incoming)
                    .map(|idx| &self.g[idx])
                    .collect();
                let node = &self.g[node_idx];

                // Mount root nodes (nodes with no predecessors)
                if predecessors.is_empty() {
                    if let Err(e) = mount(&self.srcdir, &self.sandbox, &node.pathbuf()) {
                        eprintln!("Failed to mount {}: {}", node.id(), e);
                        return;
                    }
                }

                node.build(&self.sandbox, &predecessors);
            });

            // Mark as built
            for idx in ready {
                built.insert(idx);
            }
        }
    }
}
