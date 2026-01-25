use colored::Colorize;
use log::info;
use petgraph::Direction;
use petgraph::Graph;
use petgraph::graph::{EdgeIndex, NodeIndex};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

// ANCHOR: buildtype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GNodeStatus {
    Initial,
    Mounted,
    MountedFailed,
    Running,
    Build,
    BuildFailed,
    AncestorFailed,
}
// ANCHOR_END: buildtype

#[derive(Debug)]
pub enum GraphError {
    DuplicateId(String),
    DuplicatePathBuf(PathBuf),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::DuplicateId(id) => write!(f, "Duplicate node id: {}", id),
            GraphError::DuplicatePathBuf(path) => write!(f, "Duplicate node pathbuf: {:?}", path),
        }
    }
}

impl std::error::Error for GraphError {}

pub trait GNode: Send + Sync {
    fn build(&self, _sandbox: &PathBuf, _predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
        panic!("build not implemented for {}", self.id())
    }
    fn scan(
        &self,
        _srcdir: &PathBuf,
        _predecessors: &[&Box<dyn GNode + Send + Sync>],
    ) -> Vec<PathBuf> {
        Vec::new()
    }
    fn id(&self) -> String;
    fn tag(&self) -> String;
    fn pathbuf(&self) -> PathBuf;
}

pub trait GRootNode {
    fn id(&self) -> String;
    fn tag(&self) -> String;
    fn pathbuf(&self) -> PathBuf;
}

impl<T: GRootNode + Send + Sync> GNode for T {
    fn build(&self, _sandbox: &PathBuf, _predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
        true
    }

    fn id(&self) -> String {
        GRootNode::id(self)
    }

    fn tag(&self) -> String {
        GRootNode::tag(self)
    }

    fn pathbuf(&self) -> PathBuf {
        GRootNode::pathbuf(self)
    }
}

pub struct G {
    pub srcdir: PathBuf,
    pub sandbox: PathBuf,
    pub g: Graph<Box<dyn GNode + Send + Sync>, ()>,
    pub nodes_status: HashMap<NodeIndex, GNodeStatus>,
}

impl G {
    pub fn new(srcdir: PathBuf, sandbox: PathBuf) -> Self {
        Self {
            srcdir,
            sandbox,
            g: Graph::new(),
            nodes_status: HashMap::new(),
        }
    }

    fn check_duplicate(&self, id: &str, pathbuf: &PathBuf) -> Result<(), GraphError> {
        for node in self.g.node_weights() {
            if node.id() == id {
                return Err(GraphError::DuplicateId(id.to_string()));
            }
            if node.pathbuf() == *pathbuf {
                return Err(GraphError::DuplicatePathBuf(pathbuf.clone()));
            }
        }
        Ok(())
    }

    pub fn add_node<N: GNode + Send + Sync + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeIndex, GraphError> {
        self.check_duplicate(&node.id(), &node.pathbuf())?;
        let idx = self.g.add_node(Box::new(node));
        self.nodes_status.insert(idx, GNodeStatus::Initial);
        Ok(idx)
    }

    pub fn add_root_node<N: GRootNode + Send + Sync + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeIndex, GraphError> {
        self.check_duplicate(&GNode::id(&node), &GNode::pathbuf(&node))?;
        let idx = self.g.add_node(Box::new(node));
        self.nodes_status.insert(idx, GNodeStatus::Initial);
        Ok(idx)
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        self.g.add_edge(from, to, ())
    }

    pub(crate) fn add_scanned_edges(&mut self) {
        let node_indices: Vec<NodeIndex> = self.g.node_indices().collect();

        for node_idx in node_indices {
            // Get predecessors for this node
            let predecessors: Vec<&Box<dyn GNode + Send + Sync>> = self
                .g
                .neighbors_directed(node_idx, Direction::Incoming)
                .map(|idx| &self.g[idx])
                .collect();

            // Call scan
            let scanned_paths = self.g[node_idx].scan(&self.srcdir, &predecessors);

            // For each scanned path, find the node and create an edge
            for path in scanned_paths {
                // Find node with this pathbuf
                let found_node = self
                    .g
                    .node_indices()
                    .find(|&idx| self.g[idx].pathbuf() == path);

                if let Some(from_idx) = found_node {
                    self.g.add_edge(from_idx, node_idx, ());
                }
            }
        }
    }

    pub fn print_status(&self) {
        let mut counts: HashMap<GNodeStatus, usize> = HashMap::new();

        for status in self.nodes_status.values() {
            *counts.entry(*status).or_insert(0) += 1;
        }

        let total: usize = counts.values().sum();
        let node_count = self.g.node_count();
        assert_eq!(
            total, node_count,
            "Status count mismatch: {} statuses but {} nodes",
            total, node_count
        );

        info!(
            "I:{} M:{} MF:{} R:{} B:{} BF:{} AF:{}",
            counts
                .get(&GNodeStatus::Initial)
                .unwrap_or(&0)
                .to_string()
                .bright_yellow()
                .bold(),
            counts
                .get(&GNodeStatus::Mounted)
                .unwrap_or(&0)
                .to_string()
                .bright_green()
                .bold(),
            counts.get(&GNodeStatus::MountedFailed).unwrap_or(&0),
            counts
                .get(&GNodeStatus::Running)
                .unwrap_or(&0)
                .to_string()
                .bright_cyan()
                .bold(),
            counts.get(&GNodeStatus::Build).unwrap_or(&0),
            counts.get(&GNodeStatus::BuildFailed).unwrap_or(&0),
            counts.get(&GNodeStatus::AncestorFailed).unwrap_or(&0)
        );
    }
}
