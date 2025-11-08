pub mod model;
use petgraph::Graph as G;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::model as M;

#[derive(Clone)]
pub struct A {
    pub i: u32,
}

impl M::XXX for A {
    fn tag(&self) -> String {
        println!("get tag");
        "A".to_owned()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    {
        let mut g = M::G::new();

        let anodes: Vec<A> = (0..20).map(|i| A { i }).collect::<Vec<_>>();

        // let name = "xxx".to_string();

        let n0 = A { i: 0 };
        // g.g.add_node(n0);
        // (&mut g).add_node(&n0)?;

        // M::add_node(&mut g, &n0)?;

        for node in anodes.iter() {
            let pathbuf = PathBuf::from(format!("{}.json", &node.i));
            M::add_node(&mut g, node, pathbuf)?;
        }

        let mut set: JoinSet<()> = JoinSet::new();

        for nid in (&g.g).node_indices() {
            let node = (&g.g).node_weight(nid).unwrap();
            let xnode = Arc::new(node);
            set.spawn(async move {
                let x = xnode.clone();
                x.b.tag();
                ()
            });
        }

        // g.add_node(&anodes.get(0).unwrap())?;

        // for n in anodes {
        //     &g.add_node(&n)?;
        // }

        // let _ = M::run()?;
    }
    Ok(())
}
