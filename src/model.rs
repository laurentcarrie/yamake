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
    ScanIncomplete,
    Running,
    BuildSuccess,
    BuildNotRequired,
    BuildFailed,
    AncestorFailed,
}
// ANCHOR_END: buildtype

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    Explicit,
    Scanned,
    Expanded,
}

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
    #[serde(default)]
    pub expanded: bool,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeOutput {
    pub nodes: Vec<OutputInfo>,
}

#[derive(Debug)]
pub enum GraphError {
    DuplicatePathBuf(PathBuf),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::DuplicatePathBuf(path) => write!(f, "Duplicate node pathbuf: {path:?}"),
        }
    }
}

impl std::error::Error for GraphError {}

/// An edge between two nodes
pub struct Edge {
    pub nfrom: Box<dyn GNode + Send + Sync>,
    pub nto: Box<dyn GNode + Send + Sync>,
}

/// Return type for expand method: (nodes_to_add, edges_to_add)
pub type ExpandResult = (Vec<Box<dyn GNode + Send + Sync>>, Vec<Edge>);

// ANCHOR: GNode
pub trait GNode: Send + Sync {
    fn build(&self, _sandbox: &Path, _predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        panic!("build not implemented for {}", self.pathbuf().display())
    }
    fn scan(
        &self,
        _sandbox: &Path,
        _predecessors: &[&(dyn GNode + Send + Sync)],
    ) -> (bool, Vec<PathBuf>) {
        (true, Vec::new())
    }
    fn expand(
        &self,
        _sandbox: &Path,
        _predecessors: &[&(dyn GNode + Send + Sync)],
    ) -> ExpandResult {
        (Vec::new(), Vec::new())
    }
    fn tag(&self) -> String;
    fn pathbuf(&self) -> PathBuf;
}
// ANCHOR_END: GNode

// ANCHOR: GRootNode
pub trait GRootNode {
    fn expand(
        &self,
        _sandbox: &Path,
        _predecessors: &[&(dyn GNode + Send + Sync)],
    ) -> ExpandResult {
        (Vec::new(), Vec::new())
    }
    fn tag(&self) -> String;
    fn pathbuf(&self) -> PathBuf;
}
// ANCHOR_END: GRootNode

impl<T: GRootNode + Send + Sync> GNode for T {
    fn build(&self, _sandbox: &Path, _predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        true
    }

    fn expand(&self, sandbox: &Path, predecessors: &[&(dyn GNode + Send + Sync)]) -> ExpandResult {
        GRootNode::expand(self, sandbox, predecessors)
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
    pub g: Graph<Box<dyn GNode + Send + Sync>, EdgeType>,
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

    fn check_duplicate(&self, pathbuf: &PathBuf) -> Result<(), GraphError> {
        for node in self.g.node_weights() {
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
        self.check_duplicate(&node.pathbuf())?;
        let idx = self.g.add_node(Box::new(node));
        self.nodes_status.insert(idx, GNodeStatus::Initial);
        Ok(idx)
    }

    pub fn add_root_node<N: GRootNode + Send + Sync + 'static>(
        &mut self,
        node: N,
    ) -> Result<NodeIndex, GraphError> {
        self.check_duplicate(&GNode::pathbuf(&node))?;
        let idx = self.g.add_node(Box::new(node));
        self.nodes_status.insert(idx, GNodeStatus::Initial);
        Ok(idx)
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        self.g.add_edge(from, to, EdgeType::Explicit)
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

    /// Generates a Mermaid flowchart representation of the graph.
    ///
    /// The output can be rendered by Mermaid-compatible tools.
    /// Nodes are labeled with their tag and filename, and styled based on status.
    pub fn to_mermaid(&self) -> String {
        let mut lines = Vec::new();
        lines.push("flowchart LR".to_string());

        // Create node definitions with styling
        for node_idx in self.g.node_indices() {
            let node = &self.g[node_idx];
            let tag = node.tag();
            let filename = node
                .pathbuf()
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| node.pathbuf().to_string_lossy().to_string());

            let node_id = format!("n{}", node_idx.index());
            let label = format!("{tag}<br>{filename}");

            // Choose shape based on whether node has predecessors (root vs non-root)
            let has_predecessors = self
                .g
                .neighbors_directed(node_idx, Direction::Incoming)
                .next()
                .is_some();

            let shape = if has_predecessors {
                format!("{node_id}[[\"{label}\"]];") // Stadium shape for built nodes
            } else {
                format!("{node_id}([\"{label}\"])") // Rounded for root nodes
            };

            lines.push(format!("    {shape}"));
        }

        // Add style classes based on status
        let mut style_lines: Vec<String> = Vec::new();
        for node_idx in self.g.node_indices() {
            let node_id = format!("n{}", node_idx.index());
            let status = self.nodes_status.get(&node_idx);

            let style = match status {
                Some(GNodeStatus::Initial) => "fill:#f9f9f9,stroke:#999",
                Some(GNodeStatus::MountedChanged) => "fill:#90EE90,stroke:#228B22",
                Some(GNodeStatus::MountedNotChanged) => "fill:#ADD8E6,stroke:#4169E1",
                Some(GNodeStatus::MountedFailed) => "fill:#FFB6C1,stroke:#DC143C",
                Some(GNodeStatus::ScanIncomplete) => "fill:#FFFACD,stroke:#DAA520",
                Some(GNodeStatus::Running) => "fill:#87CEEB,stroke:#00CED1",
                Some(GNodeStatus::BuildSuccess) => "fill:#98FB98,stroke:#32CD32",
                Some(GNodeStatus::BuildNotRequired) => "fill:#E6E6FA,stroke:#9370DB",
                Some(GNodeStatus::BuildFailed) => "fill:#FF6347,stroke:#B22222",
                Some(GNodeStatus::AncestorFailed) => "fill:#FFA07A,stroke:#FF4500",
                None => "fill:#fff,stroke:#333",
            };

            style_lines.push(format!("    style {node_id} {style}"));
        }

        // Add edges with different styles based on EdgeType
        let mut edge_index = 0;
        let mut link_styles: Vec<String> = Vec::new();
        for edge in self.g.edge_indices() {
            if let Some((from, to)) = self.g.edge_endpoints(edge) {
                let from_id = format!("n{}", from.index());
                let to_id = format!("n{}", to.index());
                let edge_type = self.g.edge_weight(edge);

                let arrow = match edge_type {
                    Some(EdgeType::Explicit) => "-->",
                    Some(EdgeType::Scanned) => "-.->",
                    Some(EdgeType::Expanded) => "==>",
                    None => "-->",
                };

                lines.push(format!("    {from_id} {arrow} {to_id}"));

                // Add link style for color
                let color = match edge_type {
                    Some(EdgeType::Explicit) => "#333",
                    Some(EdgeType::Scanned) => "#2196F3",
                    Some(EdgeType::Expanded) => "#FF9800",
                    None => "#333",
                };
                link_styles.push(format!(
                    "    linkStyle {edge_index} stroke:{color},stroke-width:2px"
                ));
                edge_index += 1;
            }
        }

        // Add styles at the end
        lines.extend(style_lines);
        lines.extend(link_styles);

        lines.join("\n")
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
            "I:{} MC:{} MN:{} MF:{} SI:{} R:{} BS:{} BNR:{} BF:{} AF:{}",
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
            counts.get(&GNodeStatus::ScanIncomplete).unwrap_or(&0),
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
