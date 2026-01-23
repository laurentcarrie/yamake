use std::path::PathBuf;
use std::fmt;
use std::fs;
use std::io;
use petgraph::Graph;
use petgraph::graph::{NodeIndex, EdgeIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GNodeStatus {
    Initial,
    Mounted,
    MountedFailed,
    Build,
    BuildFailed,
    AncestorFailed,
}

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
    fn build(&self, sandbox: &PathBuf, predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool;
    fn id(&self) -> String;
    fn tag(&self) -> String;
    fn pathbuf(&self) -> PathBuf;
}

pub trait GRootNode: GNode {
    fn id(&self) -> String;
    fn tag(&self) -> String;
    fn pathbuf(&self) -> PathBuf;
}

impl<T: GRootNode> GNode for T {
    fn build(&self, _sandbox: &PathBuf, _predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool { true }

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
}

impl G {
    pub fn new(srcdir: PathBuf, sandbox: PathBuf) -> Self {
        Self {
            srcdir,
            sandbox,
            g: Graph::new(),
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

    pub fn add_node<N: GNode + Send + Sync + 'static>(&mut self, node: N) -> Result<NodeIndex, GraphError> {
        self.check_duplicate(&node.id(), &node.pathbuf())?;
        Ok(self.g.add_node(Box::new(node)))
    }

    pub fn add_root_node<N: GRootNode + Send + Sync + 'static>(&mut self, node: N) -> Result<NodeIndex, GraphError> {
        self.check_duplicate(&GNode::id(&node), &GNode::pathbuf(&node))?;
        Ok(self.g.add_node(Box::new(node)))
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        self.g.add_edge(from, to, ())
    }
}

pub fn mount(srcdir: &PathBuf, sandbox: &PathBuf, p: &PathBuf) -> io::Result<()> {
    let src_path = srcdir.join(p);
    let dest_path = sandbox.join(p);

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&src_path, &dest_path)?;
    Ok(())
}
