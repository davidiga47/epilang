mod expression;
mod token;
mod parser;

use expression::Var;
use expression::Const;
use expression::Const::Integer;
use expression::Exp;

fn main() {
    let x = Var{name: String::from("x")};

    let c = Exp::Const(Integer(3));
    let ass = Exp::Assign(x, Box::new(c));

    let p = Exp::Decl(Var{name: String::from("x")}, Box::new(ass));

    println!("{}", exp_to_string(p));
}

fn const_to_string(c: Const) -> String {
    match c {
        Const::Integer(i) => i.to_string(),
        Const::Boolean(b) => b.to_string(),
        Const::String(s) => format!("\"{}\"", s),
        Const::None => String::from("None")
    }
}

fn exp_to_string(exp: Exp) -> String {
    match exp {
        Exp::Const(c) => const_to_string(c),
        Exp::Var(x) => x.name,
        Exp::Decl(x, e) => format!("let {};\n{}", x.name, exp_to_string(*e)),
        Exp::Assign(x, e) => format!("{} = {}", x.name, exp_to_string(*e)),
        Exp::Conc(e1, e2) => format!("{};\n {}", exp_to_string(*e1), exp_to_string(*e2)),
        Exp::Sum(e1, e2) => format!("{} + {}", exp_to_string(*e1), exp_to_string(*e2)),
        Exp::Sub(e1, e2) => format!("{} - {}", exp_to_string(*e1), exp_to_string(*e2)),
        Exp::Mul(e1, e2) => format!("{} * {}", exp_to_string(*e1), exp_to_string(*e2)),
        Exp::Div(e1, e2) => format!("{} / {}", exp_to_string(*e1), exp_to_string(*e2))
    }
}


