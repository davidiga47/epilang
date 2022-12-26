use std::fmt;

pub enum Exp {
    // Eg: 1, False, None, "Hello"
    Const(Const),
    // Eg: x, y, z
    Var(Var),
    // Eg: let x = y
    Decl(Var, Box<Exp>, Box<Exp>),
    // Eg: x = 3
    Assign(Var, Box<Exp>),
    // Eg: x; y
    Seq(Box<Exp>, Box<Exp>),
    // Eg: x + y
    Sum(Box<Exp>, Box<Exp>),
    // Eg: x - y
    Sub(Box<Exp>, Box<Exp>),
    // Eg: x * y
    Mul(Box<Exp>, Box<Exp>),
    // Eg: x / y
    Div(Box<Exp>, Box<Exp>),
    // Eg: x < y
    Lt(Box<Exp>, Box<Exp>),
    // Eg: x > y
    Gt(Box<Exp>, Box<Exp>),
    // Eg: x == y
    Eq(Box<Exp>, Box<Exp>),
    // Eg: x && y
    And(Box<Exp>, Box<Exp>),
    // Eg: x || y
    Or(Box<Exp>, Box<Exp>),
    // Eg: !x
    Not(Box<Exp>),
    // If then else. Eg: if e {e1} else {e2}
    IfThenElse(Box<Exp>, Box<Exp>, Box<Exp>)
}

pub struct Var {
    pub name: String,
    pub scope: usize
}

#[derive(Clone)]
pub enum Const {
    Integer(i32),
    Boolean(bool),
    String(String),
    None
}

impl fmt::Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Const::Integer(i) => write!(f, "{}", i),
            Const::Boolean(b) => write!(f, "{}", b),
            Const::String(s) => write!(f, "{}", s),
            Const::None => write!(f, "unit")
        }
    }
}
