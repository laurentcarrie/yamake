use futures::future::BoxFuture;
use petgraph::Graph;
use std::path::PathBuf;

use petgraph::graph::NodeIndex;

pub type BuildFnx = Box<
    dyn Fn(
        PathBuf,
        Vec<(PathBuf, String)>,
    ) -> BoxFuture<'static, Result<bool, Box<dyn std::error::Error>>>,
>;
type StoredFn = Box<dyn Fn(i32, i32) -> BoxFuture<'static, i32>>;

pub type BuildFn =
    fn(PathBuf, PathBuf, Vec<(PathBuf, String)>) -> Result<bool, Box<dyn std::error::Error>>;

pub fn convert_fn<
    Fut: Future<Output = Result<bool, Box<dyn std::error::Error>>> + Send + 'static,
>(
    f: impl Fn(PathBuf, Vec<(PathBuf, String)>) -> Fut + 'static,
) -> BuildFnx {
    Box::new(move |a, b| Box::pin(f(a, b)))
}

fn do_nothing(
    _sandbox: PathBuf,
    _target: PathBuf,
    _sources: Vec<(PathBuf, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}

// pub type MessageProcessor = fn(&str, mpsc::Sender<String>) -> BoxFuture<'static, ()>;

// #[derive(Debug)]
pub struct N {
    pub target: PathBuf,
    pub tag: String,
    pub build: BuildFn,
    // pub build: BuildFn,
}

impl std::fmt::Debug for N {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("target", &self.target)
            .field("tag", &self.tag)
            .finish()
    }
}

#[derive(Debug)]
pub struct E;

pub struct G {
    pub srcdir: PathBuf,
    pub sandbox: PathBuf,
    pub g: petgraph::Graph<N, E>,
}

impl G {
    pub fn new(srcdir: PathBuf, sandbox: PathBuf) -> G {
        let g: Graph<N, E> = Graph::new();
        G { g, srcdir, sandbox }
    }

    pub fn add_node(
        &mut self,
        target: PathBuf,
        tag: String,
        build: BuildFn,
    ) -> Result<NodeIndex, Box<dyn std::error::Error>> {
        let ni = self.g.try_add_node(N { target, tag, build })?;
        Ok(ni)
    }

    pub fn add_root_node(
        &mut self,
        target: PathBuf,
        tag: String,
    ) -> Result<NodeIndex, Box<dyn std::error::Error>> {
        let ni = self.g.try_add_node(N {
            target,
            tag,
            build: do_nothing,
        })?;
        Ok(ni)
    }

    pub fn add_edge(
        &mut self,
        nito: NodeIndex,
        nifrom: NodeIndex,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.g.try_add_edge(nifrom, nito, E)?;
        Ok(())
    }
}

pub enum LogItem {}

// #[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Clone)]
pub enum BuildType {
    Rebuilt(PathBuf),
    NotTouched(PathBuf),
    AncestorFailed,
    Failed,
}
