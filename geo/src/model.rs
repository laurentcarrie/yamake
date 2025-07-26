pub trait TQuadrilatere {
    fn a(&self) -> f64;
    fn b(&self) -> f64;
    fn diag(&self) -> f64;
}

pub trait TRectangle {
    fn a(&self) -> f64;
    fn b(&self) -> f64;
}

impl<T: TRectangle> TQuadrilatere for T {
    fn a(&self) -> f64 {
        self.a()
    }
    fn b(&self) -> f64 {
        self.b()
    }
    fn diag(&self) -> f64 {
        let a = self.a();
        let b = self.b();
        (a * a + b * b).sqrt()
    }
}

pub struct Rectangle {
    pub a: f64,
    pub b: f64,
}

impl TRectangle for Rectangle {
    fn a(&self) -> f64 {
        self.a
    }
    fn b(&self) -> f64 {
        self.b
    }
}

pub trait TSquare: TRectangle {
    fn a(&self) -> f64;
}

impl<T: TSquare> TRectangle for T {
    fn a(&self) -> f64 {
        TSquare::a(self)
    }
    fn b(&self) -> f64 {
        TSquare::a(self)
    }
}

pub struct Square {
    pub taille_du_cote: f64,
}

impl TSquare for Square {
    fn a(&self) -> f64 {
        self.taille_du_cote
    }
}

// impl TRectangle for Square {
//     fn a(&self) -> f64 {
//         self.taille_du_cote
//     }
//     fn b(&self) -> f64 {
//         self.taille_du_cote
//     }
// }

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

pub trait TNamedQuadrilatere: TQuadrilatere + THasName {}

impl<T: TQuadrilatere + THasName> TNamedQuadrilatere for T {}

// impl TNamedQuadrilatere for Square {}
// impl TNamedQuadrilatere for Rectangle {}

pub fn my_print<T: TNamedQuadrilatere + ?Sized>(q: &T) {
    print!(
        "type : {} ; a is {} ; b is {} ; diag is {}\n",
        // <T as THasName>::name(),
        "foo",
        q.a(),
        q.b(),
        q.diag()
    );
}
