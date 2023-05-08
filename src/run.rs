use std::collections::HashMap;
use std::ops::Add;

use rustyline::error::ReadlineError;
use rustyline::{Editor};

use crate::lexer::tokenize;
use crate::parser::parse;
use crate::semantics::eval;
use crate::parser::{parse_tokens, FunctionScope};
use crate::semantics::{eval_expression};
use crate::value::{Value, StackValue, V};

use crate::expression::Exp;
use crate::expression::Const;
use crate::expression::Var;

use crate::token::Token;
use crate::token::Operand;
use crate::token::Operator;

use std::fs;

pub fn run_file(file_path: String) {
    let text = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    //let textline = text.replace("\n", " ");
    println!("{}", text);
    run_string(text)
}

pub fn run_string(text: String) {
    let mut tokens: Vec<Token> = tokenize(text).unwrap_or_else(|err| {
        panic!("TokenizerError")
    });

    // Parse tokens to exp
    let exp: Exp = parse(&mut tokens).unwrap_or_else(|err| {
        panic!("ParserError: {}", err.msg)
    });

    println!("{}", exp_to_string(&exp));
    println!("########");

    // Evaluate expression
    let val = eval(&exp).unwrap_or_else(|err| {
        panic!("RuntimeError: {}", err.msg)
    });

    println!("Result: {}", val);
}

fn const_to_string(c: &Const) -> String {
    match c {
        Const::Integer(i) => i.to_string(),
        Const::Boolean(b) => b.to_string(),
        Const::String(s) => format!("\"{}\"", s),
        Const::None => String::from("None")
    }
}

fn var_to_string(var: &Var) -> String {
    format!("{}_{}", var.name, var.scope)
}

fn vars_to_string(vars: &Vec<Var>) -> String {
    let names: Vec<String> = vars.iter().map(|var| {var_to_string(var)}).collect();
    format!("{}", names.join(", "))
}

fn exp_to_string(exp: &Exp) -> String {
    match exp {
        Exp::Const(c) => const_to_string(c),
        Exp::Var(x) => var_to_string(x),
        Exp::List(list) => {
            let mut s = String::from("[");
            for exp in list {
                s.push_str(exp_to_string(exp).as_str())
            };
            s.push_str("]\n");
            s
        },
        Exp::ListSelection(list, index) => format!("{}[{}]", exp_to_string(list), exp_to_string(index)),
        Exp::Decl(x, val, scope) => format!("let {} = {};\n{}", var_to_string(x), exp_to_string(val), exp_to_string(scope)),
        Exp::Function(args, body) => format!("fn ({}){{\n{}\n}}", vars_to_string(args), exp_to_string(body)),
        Exp::Assign(lexp, rexp) => format!("{} = {}", exp_to_string(lexp), exp_to_string(rexp)),
        Exp::Seq(e1, e2) => format!("{};\n{}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Sum(e1, e2) => format!("{} + {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Sub(e1, e2) => format!("{} - {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Mul(e1, e2) => format!("{} * {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Div(e1, e2) => format!("{} / {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Mod(e1, e2) => format!("{} % {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Lt(e1, e2) => format!("{} < {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Lte(e1, e2) => format!("{} <= {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Gt(e1, e2) => format!("{} > {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Gte(e1, e2) => format!("{} >= {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Eq(e1, e2) => format!("{} == {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Neq(e1, e2) => format!("{} != {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::And(e1, e2) => format!("{} && {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Or(e1, e2) => format!("{} || {}", exp_to_string(e1), exp_to_string(e2)),
        Exp::Not(e) => format!("!{}", exp_to_string(e)),
        Exp::While(guard, exp) => format!("while {} {{ {} }}", exp_to_string(guard), exp_to_string(exp)),
        Exp::IfThenElse(e, e1, e2) => format!("if {} {{ {} }} else {{ {} }}", exp_to_string(e), exp_to_string(e1), exp_to_string(e2)),
        Exp::FunctionCall(e, args) => format!("{}({})", exp_to_string(e), args_to_string(args)),
        Exp::Throw(e) => format!("throw {}",exp_to_string(e)),
        Exp::Throwcc(k,e) => format!("throw {} {}",exp_to_string(k),exp_to_string(e)),
        Exp::TryCatch(e1,exc,e2) => format!("try {{ {} }} catch ( {} ) {{ {} }}",exp_to_string(e1),exp_to_string(exc),exp_to_string(e2)),
        Exp::Callcc(k,e) => format!("callcc {} in {}",var_to_string(k),exp_to_string(e))
    }
}

fn args_to_string(args: &Vec<Exp>) -> String {
    args.iter().map(|exp| {exp_to_string(exp)}).reduce(|mut a, b| {
        a.push_str(&format!(", {}", b));
        return a;
    })
    .unwrap_or(String::from(" "))
}
