use futures::future::BoxFuture;
use petgraph::Graph;
use std::path::PathBuf;

// use petgraph::graph::NodeIndex;

pub type BuildFn = Box<dyn Fn(PathBuf,Vec<(PathBuf,String)>) -> BoxFuture<'static, bool>>;
// type StoredFn = Box<dyn Fn(i32, i32) -> BoxFuture<'static, i32>>;

pub fn convert_fn<Fut: Future<Output = bool> + Send + 'static>(
    f: impl Fn(PathBuf,Vec<(PathBuf,Sting)>) -> Fut + 'static,
) -> BuildFn {
    Box::new(move |a,b| Box::pin(f(a,b)))
}

// pub type MessageProcessor = fn(&str, mpsc::Sender<String>) -> BoxFuture<'static, ()>;

// #[derive(Debug)]
pub struct N {
    target: PathBuf,
    // sources: Vec<(PathBuf, String)>,
    // build: fn(
    //     target: PathBuf,
    //     sources: Vec<(PathBuf, String)>,
    // ) -> Result<bool, Box<dyn std::error::Error>>,
    build: BuildFn,
}

impl std::fmt::Debug for N {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node").finish()
    }
}

#[derive(Debug)]
pub struct E;

pub struct G {
    pub g: petgraph::Graph<N, E>,
}

// async fn do_nothing(
//     _target: PathBuf,
//     _sources: Vec<(PathBuf, String)>,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     Ok(true)
// }

impl G {
    pub fn new() -> G {
        let g: Graph<N, E> = Graph::new();
        G { g }
    }

    pub fn add_node(
        &mut self,
        target: PathBuf,
        build: BuildFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ni = self.g.try_add_node(N {
            target,
            // sources,
            // build: nt.build,
            build,
        })?;
        Ok(())
    }
}
