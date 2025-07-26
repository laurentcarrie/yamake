use petgraph::Graph;
use std::path::PathBuf;

use petgraph::graph::NodeIndex;

pub struct NodeType {
    pub build: fn(
        target: PathBuf,
        sources: Vec<(PathBuf, String)>,
    ) -> dyn Future<Output = Result<bool, Box<dyn std::error::Error>>>,
    // fn scan(sources:Vec<PathBuf>) -> Result<bool,Box<dyn std::error::Error>>;
}

pub struct N {
    target: PathBuf,
    sources: Vec<(PathBuf, String)>,
    build: fn(
        target: PathBuf,
        sources: Vec<(PathBuf, String)>,
    ) -> dyn Future<Output = Result<bool, Box<dyn std::error::Error>>>,
}

pub struct E;

pub struct G {
    g: petgraph::Graph<N, E>,
}

async fn do_nothing(
    _target: PathBuf,
    _sources: Vec<(PathBuf, String)>,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}

impl G {
    pub fn new() -> G {
        let g: Graph<N, E> = Graph::new();
        G { g }
    }

    // pub fn add_source_node(
    //     &mut self,
    //     target: PathBuf,
    // ) -> Result<NodeIndex, Box<dyn std::error::Error>> {
    //     let ni = self.g.try_add_node(N {
    //         target,
    //         sources: vec![],
    //         build: |_, _| box<dyn {Box<dyn Box::new(true)>}
    //     })?;
    //     Ok(ni)
    // }

    pub fn add_node(
        &mut self,
        target: PathBuf,
        sources: Vec<(NodeIndex, String)>,
        nt: NodeType,
    ) -> Result<NodeIndex, Box<dyn std::error::Error>> {
        let sources: Vec<(PathBuf, String)> = sources
            .iter()
            .map(|(ni, s)| match self.g.node_weight(*ni) {
                Some(p) => {
                    let x: (PathBuf, String) = (p.target.clone(), s.clone());
                    Ok(x)
                }
                None => Err("huh, did not find node in graph"),
            })
            .collect::<Result<_, _>>()?;

        let ni = self.g.try_add_node(N {
            target,
            sources,
            build: nt.build,
        })?;

        Ok(ni)
    }
}
