use petgraph as P;
use std::collections::HashMap;

pub trait TQuadrilatere {
    fn a(&self) -> u64;
    fn b(&self) -> u64;
    fn diag(&self) -> u64;
}

pub trait TRectangle {
    fn a(&self) -> u64;
    fn b(&self) -> u64;
}

impl<T: TRectangle> TQuadrilatere for T {
    fn a(&self) -> u64 {
        self.a()
    }
    fn b(&self) -> u64 {
        self.b()
    }
    fn diag(&self) -> u64 {
        let a = self.a() as f64;
        let b = self.b() as f64;
        let ret = (a * a + b * b).sqrt();
        ret as u64
    }
}
#[derive(Clone, Debug, Hash, Copy)]
pub struct Rectangle {
    pub a: u64,
    pub b: u64,
}

// impl Eq for Rectangle {}

// impl PartialEq for Rectangle {
//     fn eq(&self,other:&self) -> bool {
//         if self.a > other.a || self.a < other.a or self.b > other.b or self.b < other.b {
//             false
//         }
//         true
//     }
// }

impl TRectangle for Rectangle {
    fn a(&self) -> u64 {
        self.a
    }
    fn b(&self) -> u64 {
        self.b
    }
}

pub trait TSquare: TRectangle {
    fn a(&self) -> u64;
}

impl<T: TSquare> TRectangle for T {
    fn a(&self) -> u64 {
        TSquare::a(self)
    }
    fn b(&self) -> u64 {
        TSquare::a(self)
    }
}
#[derive(Clone, Debug, Copy)]

pub struct Square {
    pub taille_du_cote: u64,
}

impl TSquare for Square {
    fn a(&self) -> u64 {
        self.taille_du_cote
    }
}

pub trait THasName {
    fn name(&self) -> String;
}

impl THasName for Square {
    fn name(&self) -> String {
        "Square".to_string()
    }
}

impl THasName for Rectangle {
    fn name(&self) -> String {
        "Rectangle".to_string()
    }
}

pub trait TNamedQuadrilatere: TQuadrilatere + THasName + std::fmt::Debug {}

impl<T: TQuadrilatere + THasName + Sized + std::fmt::Debug + std::fmt::Debug> TNamedQuadrilatere
    for T
{
}

// impl TNamedQuadrilatere for Square {}
// impl TNamedQuadrilatere for Rectangle {}

pub fn my_print<T: TNamedQuadrilatere + ?Sized>(q: &T) {
    print!(
        "type : {} ; a is {} ; b is {} ; diag is {}\n",
        <T as THasName>::name(q),
        q.a(),
        q.b(),
        q.diag()
    );
}

pub struct NG {}
#[derive(Debug)]
pub struct EG {}

pub trait TN {}
pub struct MyGraph {
    nodes: Vec<Arc<dyn TNamedQuadrilatere>>,
    // nodes: HashMap<N, P::graph::NodeIndex>,
    //indices: HashMap<P::graph::NodeIndex, N>,
    pub g: P::graph::Graph<Arc<dyn TNamedQuadrilatere>, EG, P::Directed>,
}

impl MyGraph {
    // impl<N: std::fmt::Debug + ?Sized> MyGraph<N> {
    pub fn new() -> MyGraph {
        MyGraph {
            nodes: Vec::<Arc<dyn TNamedQuadrilatere>>::new(),
            // nodesh: HashMap::<N, P::graph::NodeIndex>::new(),
            g: P::graph::Graph::new(),
            // indices: HashMap::<P::graph::NodeIndex, N>::new(),
        }
    }

    // fn get_node(&self, n: &N) -> Option<&N> {
    //     self.nodes.get(n)
    // }
    pub fn add_node<T: TNamedQuadrilatere + 'static + Copy>(
        &mut self,
        n: &T,
    ) -> Result<(), Arc<dyn std::error::Error>> {
        let ni = self.g.try_add_node(Arc::new(*n))?;
        println!("--- add node {}", n.name());
        self.nodes.push(Arc::new(n.clone()));
        // self.indices.insert(ni, n.clone());
        Ok(())
    }

    // pub fn add_edge(&mut self, nfrom: N, nto: N) -> Result<(), Arc<dyn std::error::Error>> {
    //     let _ni = self.g.try_add_edge(
    //         *self.try_get_ni_from_node(&nfrom)?,
    //         *self.try_get_ni_from_node(&nto)?,
    //         EG {},
    //     )?;
    //     Ok(())
    // }
}

// impl std::PartialEq for dyn TNamedQuadrilatere {}
