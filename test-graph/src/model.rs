use petgraph::graph;
use std::path::PathBuf;
use std::sync::Arc;

pub trait XXX: Send + Sync {
    fn tag(&self) -> String;
}

pub struct N<'a> {
    pub b: &'a (dyn XXX + 'a), // pub name: String,
    pub target: PathBuf,
}

pub struct E;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub struct G<'a> {
    pub g: petgraph::Graph<N<'a>, E>,
}

impl<'a> G<'a> {
    pub fn new() -> G<'a> {
        let g = petgraph::Graph::new();
        G { g }
    }

    pub fn add_node<T: XXX + 'static>(
        &mut self,
        b: &'a T,
        target: PathBuf, // name: String,
    ) -> Result<(), Box<&dyn std::error::Error>> {
        // let node = N { b };
        // self.g.add_node(node);
        Ok(())
    }
}

pub fn add_node<'a, T: XXX + 'static>(
    g: &mut G<'a>,
    b: &'a T,
    target: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let node = N { b, target };
    g.g.add_node(node);
    Ok(())
}
