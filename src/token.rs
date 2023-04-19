use std::fmt;

use crate::expression::{Const, Exp};

pub enum Token {
    Operand(Operand),
    Operator(Operator),
    While,
    If,
    Else,
    Let,
    Fn,
    FunctionCallOpen,
    ListSelectionOpen,
    RoundBracketOpen,
    RoundBracketClosed,
    SquareBracketOpen,
    SquareBracketClosed,
    CurlyBracketOpen,
    CurlyBracketClosed,
    Comma,
    Try,
    Catch,
    /*Callcc,
    Setcc,
    In*/
}

impl Token {
    /**
     * If a token is callable, then a `(` token right after it
     * is interpreted as the beginning of a function call
     */
    pub fn is_callable(&self) -> bool {
        match self {
            Token::Operand(_) => true,
            Token::RoundBracketClosed => true,
            Token::SquareBracketClosed => true,
            Token::Operator(_) => false,
            Token::While => false,
            Token::If => false,
            Token::Else => false,
            Token::Let => false,
            Token::Fn => false,
            Token::FunctionCallOpen => false,
            Token::ListSelectionOpen => false,
            Token::RoundBracketOpen => false,
            Token::CurlyBracketOpen => false,
            Token::CurlyBracketClosed => false,
            Token::SquareBracketOpen => false,
            Token::Comma => false,
            Token::Try => false,
            Token::Catch => false,
            /*Token::Callcc => false,
            Token::Setcc => false,
            Token::In => false*/
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Operator(Operator::Assign), Token::Operator(Operator::Assign)) => true,
            (Token::Operator(Operator::Seq), Token::Operator(Operator::Seq)) => true,
            (Token::CurlyBracketOpen, Token::CurlyBracketOpen) => true,
            (Token::CurlyBracketClosed, Token::CurlyBracketClosed) => true,
            (Token::If, Token::If) => true,
            (Token::Else, Token::Else) => true,
            (Token::FunctionCallOpen, Token::FunctionCallOpen) => true,
            (Token::ListSelectionOpen, Token::ListSelectionOpen) => true,
            (Token::RoundBracketOpen, Token::RoundBracketOpen) => true,
            (Token::RoundBracketClosed, Token::RoundBracketClosed) => true,
            (Token::Let, Token::Let) => true,
            (Token::Fn, Token::Fn) => true,
            (Token::Operand(Operand::Null), Token::Operand(Operand::Null)) => true,
            (Token::Operator(Operator::Eq), Token::Operator(Operator::Eq)) => true,
            (Token::Try, Token::Try) => true,
            (Token::Catch, Token::Catch) => true,
            /*(Token::Callcc, Token::Callcc) => true,
            (Token::Setcc, Token::Setcc) => true,
            (Token::In, Token::In) => true,*/
            _ => false
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Operand(o) => write!(f, "{}", *o),
            Token::Operator(o) => write!(f, "{}", *o),
            Token::While => write!(f, "while"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            Token::FunctionCallOpen => write!(f, "("),
            Token::ListSelectionOpen => write!(f, "["),
            Token::RoundBracketOpen => write!(f, "("),
            Token::RoundBracketClosed => write!(f, ")"),
            Token::SquareBracketOpen => write!(f, "["),
            Token::SquareBracketClosed => write!(f, "]"),
            Token::CurlyBracketOpen => write!(f, "{{"),
            Token::CurlyBracketClosed => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Try => write!(f, "try"),
            Token::Catch => write!(f, "catch"),
            /*Token::Callcc => write!(f, "callcc"),
            Token::Setcc => write!(f, "setcc"),
            Token::In => write!(f,"in")*/
        }
    }
}


pub enum Operand {
    Null,
    Int(i32),
    Bool(bool),
    Str(String),
    Var(String)
}

impl Operand {
    pub fn to_exp(&self) -> Exp {
        match self {
            Operand::Null => Exp::Const(Const::None),
            Operand::Int(i) => Exp::Const(Const::Integer(*i)),
            Operand::Bool(b) => Exp::Const(Const::Boolean(*b)),
            Operand::Str(s) => Exp::Const(Const::String(s.clone())),
            Operand::Var(_) => panic!("Never call to_exp() on variables because we need to know the scope to parse them correctly"),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Null => write!(f, "null"),
            Operand::Int(i) => write!(f, "{}", *i),
            Operand::Bool(b) => write!(f, "{}", *b),
            Operand::Str(str) => write!(f, "\"{}\"", str),
            Operand::Var(name) => write!(f, "{}", name)
        }
    }
}

#[derive(Copy, Clone)]
pub enum Operator {
    Seq,
    Assign,
    And,
    Or,
    Not,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Sum,
    Sub,
    Mul,
    Div,
    Mod,
    Throw
}

impl Operator {
    pub fn precedence(&self) -> i32 {
        match self {
            Operator::Throw => 0,
            Operator::Mul => 1,
            Operator::Div => 1,
            Operator::Mod => 1,
            Operator::Sum => 2,
            Operator::Sub => 2,
            Operator::Not => 4,
            Operator::Lt => 4,
            Operator::Lte => 4,
            Operator::Gt => 4,
            Operator::Gte => 4,
            Operator::Eq => 4,
            Operator::Neq => 4,
            Operator::And => 5,
            Operator::Or => 6,
            Operator::Assign => 7,
            Operator::Seq => 8
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Mul => write!(f, "{}","*"),
            Operator::Div => write!(f, "{}","/"),
            Operator::Mod => write!(f, "{}","%"),
            Operator::Sum => write!(f, "{}","+"),
            Operator::Sub => write!(f, "{}","-"),
            Operator::Not => write!(f, "{}","!"),
            Operator::Lt => write!(f, "{}","<"),
            Operator::Lte => write!(f, "{}","<="),
            Operator::Gt => write!(f, "{}",">"),
            Operator::Gte => write!(f, "{}",">="),
            Operator::Eq => write!(f, "{}","=="),
            Operator::Neq => write!(f, "{}","!="),
            Operator::And => write!(f, "{}","&&"),
            Operator::Or => write!(f, "{}","||"),
            Operator::Assign => write!(f, "{}","="),
            Operator::Seq => write!(f, "{}",";"),
            Operator::Throw => write!(f,"{}","throw")
        }
    }
}
