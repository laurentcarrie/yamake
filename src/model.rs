use colored::Colorize;
use log::info;
use petgraph::Direction;
use petgraph::Graph;
use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

// ANCHOR: buildtype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GNodeStatus {
    Initial,
    MountedChanged,
    MountedNotChanged,
    MountedFailed,
    Running,
    BuildSuccess,
    BuildNotRequired,
    BuildFailed,
    AncestorFailed,
}
// ANCHOR_END: buildtype

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredecessorInfo {
    pub pathbuf: PathBuf,
    pub status: GNodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    pub pathbuf: PathBuf,
    pub status: GNodeStatus,
    pub digest: Option<String>,
    pub absolute_path: Option<PathBuf>,
    pub stdout_path: Option<PathBuf>,
    pub stderr_path: Option<PathBuf>,
    pub predecessors: Vec<PredecessorInfo>,
}

#[derive(Debug)]
pub enum GraphError {
    DuplicateId(String),
    DuplicatePathBuf(PathBuf),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::DuplicateId(id) => write!(f, "Duplicate node id: {id}"),
            GraphError::DuplicatePathBuf(path) => write!(f, "Duplicate node pathbuf: {path:?}"),
        }
    }
}

impl std::error::Error for GraphError {}

pub trait GNode: Send + Sync {
    fn build(&self, _sandbox: &Path, _predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        panic!("build not implemented for {}", self.id())
    }
    fn scan(
        &self,
        _srcdir: &Path,
        _predecessors: &[&(dyn GNode + Send + Sync)],
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
    fn build(&self, _sandbox: &Path, _predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
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
            let predecessors: Vec<&(dyn GNode + Send + Sync)> = self
                .g
                .neighbors_directed(node_idx, Direction::Incoming)
                .map(|idx| self.g[idx].as_ref())
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

    /// Returns all root nodes (nodes with no predecessors) in the predecessor tree of the given node.
    pub fn root_predecessors(&self, node_idx: NodeIndex) -> Vec<NodeIndex> {
        let mut visited: HashSet<NodeIndex> = HashSet::new();
        let mut to_visit: Vec<NodeIndex> = vec![node_idx];
        let mut roots: Vec<NodeIndex> = Vec::new();

        while let Some(idx) = to_visit.pop() {
            if visited.contains(&idx) {
                continue;
            }
            visited.insert(idx);

            let predecessors: Vec<NodeIndex> = self
                .g
                .neighbors_directed(idx, Direction::Incoming)
                .collect();

            if predecessors.is_empty() {
                // This is a root node
                roots.push(idx);
            } else {
                // Add predecessors to visit
                for pred in predecessors {
                    to_visit.push(pred);
                }
            }
        }

        roots
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
            "Status count mismatch: {total} statuses but {node_count} nodes"
        );

        info!(
            "I:{} MC:{} MN:{} MF:{} R:{} BS:{} BNR:{} BF:{} AF:{}",
            counts
                .get(&GNodeStatus::Initial)
                .unwrap_or(&0)
                .to_string()
                .bright_yellow()
                .bold(),
            counts
                .get(&GNodeStatus::MountedChanged)
                .unwrap_or(&0)
                .to_string()
                .bright_green()
                .bold(),
            counts
                .get(&GNodeStatus::MountedNotChanged)
                .unwrap_or(&0)
                .to_string()
                .bright_blue()
                .bold(),
            counts.get(&GNodeStatus::MountedFailed).unwrap_or(&0),
            counts
                .get(&GNodeStatus::Running)
                .unwrap_or(&0)
                .to_string()
                .bright_cyan()
                .bold(),
            counts.get(&GNodeStatus::BuildSuccess).unwrap_or(&0),
            counts
                .get(&GNodeStatus::BuildNotRequired)
                .unwrap_or(&0)
                .to_string()
                .bright_magenta()
                .bold(),
            counts.get(&GNodeStatus::BuildFailed).unwrap_or(&0),
            counts.get(&GNodeStatus::AncestorFailed).unwrap_or(&0)
        );
    }
}
