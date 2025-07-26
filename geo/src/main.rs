pub mod model;

use crate::model as M;
use crate::model::TQuadrilatere;
use crate::model::TRectangle;
use crate::model::TSquare;

fn main() {
    let r = M::Rectangle { a: 3.0, b: 4.0 };
    println!(" diag of r : {}", r.diag());

    let a = M::Square {
        taille_du_cote: 49.0,
    };
    println!(" diag of a : {}", a.diag());

    println!("Hello, world!");
    M::my_print(&a);
    M::my_print(&r);

    // let b = M::X { q: a, yyy: 42 };

    let v: Vec<Box<dyn M::TNamedQuadrilatere>> = vec![Box::new(a), Box::new(r)];
    for x in v {
        // M::my_print::<dyn M::TNamedQuadrilatere>(&x);
        println!("{}", x.a());
        println!("{}", x.name());

        M::my_print(&*x);
    }
}
