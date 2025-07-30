pub mod model;
use crate::model as M;
use crate::model::TQuadrilatere;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use std::path::PathBuf;
pub struct N {}
pub struct E {}

fn main() {
    let r = M::Rectangle { a: 3, b: 4 };
    println!(" diag of r : {}", r.diag());

    let a = M::Square { taille_du_cote: 49 };
    println!(" diag of a : {}", a.diag());

    println!("Hello, world!");
    M::my_print(&a);
    M::my_print(&r);

    // let b = M::X { q: a, yyy: 42 };

    let v: Vec<Box<dyn M::TNamedQuadrilatere>> = vec![Box::new(a), Box::new(r)];
    // let v2 = v.clone();
    // for x in v {
    //     // M::my_print::<dyn M::TNamedQuadrilatere>(&x);
    //     println!("{}", x.a());

    //     println!("{}", x.name());

    //     M::my_print(&*x);
    // }

    // let mut g: Graph<dyn M::TNamedQuadrilatere, E> = Graph::new();

    // let x = v[0];

    let mut g = M::MyGraph::new();
    g.add_node(&a).unwrap();
    g.add_node(&r).unwrap();
    g.g.add_edge(NodeIndex::new(0), NodeIndex::new(1), M::EG {});

    for ni in g.g.node_indices() {
        let n = g.g.node_weight(ni).unwrap() ;
        println!("name : {}",n.name()) ;
    }

    // for demo or debug, output the tree
    let basic_dot = Dot::new(&g.g);
    let pdot = PathBuf::from("out.dot");
    println!("write dot file");
    std::fs::write(pdot, format!("{:?}", basic_dot)).unwrap();
}
