use std::fmt;

use colored_text::Colorize;
use petgraph::Graph;
use serde::{Deserialize, Serialize};
use simple_mermaid::mermaid;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use petgraph::graph::NodeIndex;

/// the trait of a Node in the make graph
///
///

#[derive(Clone, Debug)]
pub struct PathWithTag {
    pub path: PathBuf,
    pub tag: String,
}

pub trait GNode: Send + Sync {
    /// the function that builds the target file of a node, taking the predecessor nodes as inputs
    ///
    /// sandbox: the root of the sandbox directory, you should not need it
    /// sources: the sources for the build ( the predecessor nodes in the graph )
    /// stdout: where to write stdout. Pass it to std::process::Command if you call a command
    /// stderr: where to write stderr
    ///
    // ANCHOR: build
    fn build(
        &self,
        _sandbox: PathBuf,
        _sources: Vec<PathWithTag>,
        _stdout: std::fs::File,
        _stderr: std::fs::File,
    ) -> bool {
        log::error!("cannot build node {:?}", self.target());
        false
    }
    // ANCHOR_END: build

    // ANCHOR: scan
    fn scan(
        &self,
        _srcdir: PathBuf,
        _sources: Vec<PathWithTag>,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
    // ANCHOR_END:scan

    // ANCHOR: expand
    fn expand(
        &self,
        _srcdir: PathBuf,
        _sources: Vec<PathWithTag>,
    ) -> Result<Vec<Box<dyn GNode>>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
    // ANCHOR_END:expand

    // ANCHOR: target
    // the target path of a node, relative to the sandbox path
    fn target(&self) -> PathBuf;
    // ANCHOR_END: target

    // ANCHOR: tag
    fn tag(&self) -> String;
    // ANCHOR_END: tag

    // unique id in the graph
    // ANCHOR: id
    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
    // ANCHOR_END: id
}

impl std::fmt::Debug for dyn GNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("target", &self.target())
            .field("tag", &self.tag())
            .finish()
    }
}

#[derive(Debug)]
pub enum EKind {
    Scanned,
    Explicit,
}

#[derive(Debug)]
pub struct E {
    pub kind: EKind,
}

pub struct G {
    pub srcdir: PathBuf,
    pub sandbox: PathBuf,
    // pub map: HashMap<PathBuf, NodeIndex>,
    pub g: petgraph::Graph<Arc<dyn GNode>, E>,
    pub map: HashMap<String, Box<dyn GNode>>,
    pub(crate) needs_rebuild: HashMap<String, bool>,
    // pub(crate) status: HashMap<String, EStatus>,
}

impl G {
    pub fn new(srcdir: PathBuf, sandbox: PathBuf) -> Result<G, Box<dyn std::error::Error>> {
        let g: Graph<Arc<dyn GNode>, E> = Graph::new();
        // let map: HashMap<PathBuf, NodeIndex> = HashMap::new();
        let srcdir = srcdir.canonicalize()?;
        log::info!("{}:{} ; {:?}", file!(), line!(), sandbox);
        let sandbox = sandbox.canonicalize().expect(
            "could not canonicalize sandbox path, please create it first"
                .hex("#FF1493")
                .on_hex("#F0FFFF")
                .bold()
                .as_str(),
        );
        let map = HashMap::new();
        let needs_rebuild: HashMap<String, bool> = HashMap::new();
        // let status: HashMap<String, EStatus> = HashMap::new();

        std::fs::create_dir_all(&sandbox)?;
        Ok(G {
            g,
            srcdir,
            sandbox,
            map,
            needs_rebuild,
            // status,
        })
    }

    pub fn add_node<T: GNode + 'static>(
        &mut self,
        n: T,
    ) -> Result<NodeIndex, Box<dyn std::error::Error>> {
        if self.map.contains_key(&n.id()) {
            return Err(format!("cannot add node with existing key '{}'", n.id()).into());
        }
        log::info!("add node {:?}", n.target());
        let existing = self.ni_of_path(n.target());
        if existing.is_ok() {
            let msg = format!("path already exists {:?}", n.target());
            log::error!("{msg}");
            return Err(msg.into());
        }
        let ni = self.g.try_add_node(Arc::new(n))?;
        Ok(ni)
    }

    pub fn ni_of_path(&self, p: PathBuf) -> Result<NodeIndex, Box<dyn std::error::Error>> {
        for ni in self.g.node_indices() {
            let n = self.g.node_weight(ni).ok_or(format!(
                "node {} not found",
                p.display().hex("#FF1493").on_hex("#F0FFFF").bold()
            ))?;
            if n.target() == p {
                return Ok(ni);
            }
        }
        // log::error!("{}:{}, path not found : {p:?}", file!(), line!());
        Err(format!("cannot find node index for path {p:?}").into())
    }

    pub fn add_edge(
        &mut self,
        pto: PathBuf,
        pfrom: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nito = self.ni_of_path(pto.clone())?;
        let nifrom = self.ni_of_path(pfrom.clone())?;

        log::info!("add edge {:?} => {:?}", pfrom.clone(), pto.clone());

        self.g.try_add_edge(
            nifrom,
            nito,
            E {
                kind: EKind::Explicit,
            },
        )?;
        Ok(())
    }

    pub fn is_root_node(&self, ni: NodeIndex) -> bool {
        for npred in self.g.edges_directed(ni, petgraph::Incoming) {
            match npred.weight().kind {
                EKind::Scanned => (),
                EKind::Explicit => {
                    return false;
                }
            }
        }
        true
    }
    // pub fn mount(&mut self) -> Result<u32, Box<dyn std::error::Error>> {
    //     // crate::run::make(self, true, 4, ETraverse::Scan).await?;
    //     let n = crate::run::mount(self)?;
    //     Ok(n)
    // }
    pub(crate) async fn scan(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // crate::run::make(self, true, 4, ETraverse::Scan).await?;
        crate::actions::scan::scan(self).await?;
        Ok(())
    }
    pub async fn make(
        &mut self,
        force_rebuild: bool,
        nb_workers: u32,
    ) -> Result<MakeReturn, Box<dyn std::error::Error>> {
        let ret = crate::actions::run::make(self, force_rebuild, nb_workers).await?;
        Ok(ret)
    }
}

// ANCHOR: buildtype
#[derive(PartialEq, Debug, Hash, Clone, Serialize, Deserialize)]
pub enum BuildType {
    /// the node is remounted but is different than before
    MountChanged(PathBuf),
    /// the node is remounted but is the same as before
    MountNotChanged(PathBuf),
    /// the node was rebuilt (and changed)
    Rebuilt(PathBuf),
    /// the node was rebuild, but not changed
    RebuiltButUnchanged(PathBuf),
    /// the node was not touched (because none of its deps has changed)
    NotRebuilt(PathBuf),
    /// there was a built error in one of the ancestors, therefore this node is failed
    /// (there was no attempt to rebuild it)
    AncestorFailed,
    /// building this node has failed
    Failed,
}
// ANCHOR_END: buildtype

#[derive(Debug)]
pub struct MakeReturn {
    pub success: bool,
    pub nt: HashMap<NodeIndex, BuildType>,
}

impl fmt::Display for BuildType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildType::MountChanged(_) => write!(f, "Mounted"),
            BuildType::MountNotChanged(_) => write!(f, "NotReMounted"),
            BuildType::Rebuilt(_) => write!(f, "Rebuilt"),
            BuildType::RebuiltButUnchanged(_) => write!(f, "RebuiltButUnchanged"),
            BuildType::NotRebuilt(_) => write!(f, "NotRebuilt"),
            BuildType::AncestorFailed => write!(f, "AncestorFailed"),
            BuildType::Failed => write!(f, "Failed"),
        }
    }
}
