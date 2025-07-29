use colored_text::Colorize;
use petgraph::Graph;
use std::{collections::HashMap, path::PathBuf};

use petgraph::graph::NodeIndex;

// pub type BuildFnx = Box<
//     dyn Fn(
//         PathBuf,
//         Vec<(PathBuf, String)>,
//     ) -> BoxFuture<'static, Result<bool, Box<dyn std::error::Error>>>,
// >;
// type StoredFn = Box<dyn Fn(i32, i32) -> BoxFuture<'static, i32>>;

pub type BuildFn = fn(
    PathBuf,
    petgraph::graph::NodeIndex,
    PathBuf,
    Vec<(PathBuf, String)>,
    PathBuf,
    PathBuf,
) -> Result<bool, Box<dyn std::error::Error>>;

pub type ScanFn =
    fn(PathBuf, PathBuf, PathBuf, PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>>;

// pub fn convert_fn<
//     Fut: Future<Output = Result<bool, Box<dyn std::error::Error>>> + Send + 'static,
// >(
//     f: impl Fn(PathBuf, Vec<(PathBuf, String)>) -> Fut + 'static,
// ) -> BuildFnx {
//     Box::new(move |a, b| Box::pin(f(a, b)))
// }

fn do_nothing(
    _sandbox: PathBuf,
    _ni: NodeIndex,
    _target: PathBuf,
    _sources: Vec<(PathBuf, String)>,
    _stdout: PathBuf,
    _stderr: PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}

pub fn scan_nothing(
    _srcdir: PathBuf,
    _target: PathBuf,
    _stdout: PathBuf,
    _stderr: PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    Ok(vec![])
}

// pub type MessageProcessor = fn(&str, mpsc::Sender<String>) -> BoxFuture<'static, ()>;

// #[derive(Debug)]
pub struct N {
    pub target: PathBuf,
    pub tag: String,
    pub build: BuildFn,
    pub scan: ScanFn, // pub build: BuildFn,
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
pub enum EKind {
    Scanned,
    Direct,
}

#[derive(Debug)]
pub struct E {
    pub kind: EKind,
}

pub struct G {
    pub srcdir: PathBuf,
    pub sandbox: PathBuf,
    pub map: HashMap<PathBuf, NodeIndex>,
    pub g: petgraph::Graph<N, E>,
}

impl G {
    pub fn new(srcdir: PathBuf, sandbox: PathBuf) -> Result<G, Box<dyn std::error::Error>> {
        let g: Graph<N, E> = Graph::new();
        let map: HashMap<PathBuf, NodeIndex> = HashMap::new();
        let srcdir = srcdir.canonicalize()?;
        log::info!("{}:{} ; {:?}", file!(), line!(), sandbox);
        let sandbox = sandbox.canonicalize().expect(
            "could not canonicalize sandbox path, please create it first"
                .hex("#FF1493")
                .on_hex("#F0FFFF")
                .bold()
                .as_str(),
        );
        log::info!("{}:{}", file!(), line!());

        std::fs::create_dir_all(&sandbox)?;
        Ok(G {
            g,
            srcdir,
            sandbox,
            map,
        })
    }

    pub fn add_node(
        &mut self,
        target: PathBuf,
        tag: String,
        build: BuildFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("add  node {:?}", target);

        // let target = target.canonicalize()?;
        log::info!("{}:{}", file!(), line!());

        // let target = target.strip_prefix(&self.srcdir)?.to_path_buf();
        log::info!("{}:{}", file!(), line!());

        let ni = self.g.try_add_node(N {
            target: target.clone(),
            tag,
            build,
            scan: scan_nothing,
        })?;
        self.map.insert(target.clone(), ni);
        Ok(())
    }

    pub fn add_root_node(
        &mut self,
        target: PathBuf,
        tag: String,
        scan: ScanFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let target = target.canonicalize()?;
        let target = target.strip_prefix(&self.srcdir)?.to_path_buf();
        let ni = self.g.try_add_node(N {
            target: target.clone(),
            tag,
            build: do_nothing,
            scan,
        })?;
        self.map.insert(target.clone(), ni);
        Ok(())
    }

    pub fn add_edge(
        &mut self,
        pto: PathBuf,
        pfrom: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let get = |p: PathBuf| {
            self.map.get(&p).ok_or(format!(
                "when trying to build an edge, node {} not found",
                p.display().hex("#FF1493").on_hex("#F0FFFF").bold()
            ))
        };
        self.g.try_add_edge(
            *get(pfrom.clone())?,
            *get(pto.clone())?,
            E {
                kind: EKind::Direct,
            },
        )?;
        Ok(())
    }

    pub async fn make(
        &self,
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
    NotTouched(PathBuf),
    AncestorFailed,
    Failed,
}
