use colored_text::Colorize;
use petgraph::Graph;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use petgraph::graph::NodeIndex;

/// the trait of a Node in the make graph
///

pub trait GNode: Send + Sync {
    /// the function that builds the target file of a node, taking the predecessor nodes as inputs
    ///
    /// sandbox: the root of the sandbox directory, you should not need it
    /// sources: the sources for the build ( the predecessor nodes in the graph )
    /// stdout: where to write stdout. Pass it to std::process::Command if you call a command
    /// stderr: where to write stderr
    fn build(
        &self,
        _sandbox: PathBuf,
        _sources: Vec<(PathBuf, String)>,
        _deps: Vec<PathBuf>,
        _stdout: PathBuf,
        _stderr: PathBuf,
    ) -> bool {
        panic!(
            r###"build function of node {:?} was called, but no implementation found "###,
            self.target()
        );
    }

    fn scan(
        &self,
        _srcdir: PathBuf,
        _source: PathBuf,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        panic!(
            r###"scan function of node {:?} was called, but no implementation found "###,
            self.target()
        );
    }

    fn target(&self) -> PathBuf;
    fn tag(&self) -> String;

    // unique id in the graph
    fn id(&self) -> String;
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
    Direct,
}

#[derive(Debug)]
pub struct E {
    pub kind: EKind,
}

#[derive(Debug)]
pub enum EStatus {
    Initial,
    MountedChanged,
    MountedNotChanged,
    Skipped,
    Rebuilt,
    Failed,
}

pub struct H {
    old_digest: String,
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
        Err("path not found {p:?}".into())
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
                kind: EKind::Direct,
            },
        )?;
        Ok(())
    }

    pub fn is_root_node(&self, ni: NodeIndex) -> bool {
        for npred in self.g.edges_directed(ni, petgraph::Incoming) {
            match npred.weight().kind {
                EKind::Scanned => (),
                EKind::Direct => {
                    return false;
                }
            }
        }
        true
    }

    pub async fn make(
        &mut self,
        force_rebuild: bool,
        nb_workers: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::run::make(self, force_rebuild, nb_workers).await?;
        Ok(())
    }

    pub async fn scan(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // crate::run::make(self, true, 4, ETraverse::Scan).await?;
        crate::run::scan(self).await?;
        Ok(())
    }
}

pub enum LogItem {}

// #[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Clone)]
pub enum BuildType {
    Rebuilt(PathBuf),
    RebuiltButUnchanged(PathBuf),
    NotTouched(PathBuf),
    AncestorFailed,
    Failed,
}
