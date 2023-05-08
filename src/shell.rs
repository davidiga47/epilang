use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::{Editor};

use crate::lexer::tokenize;
use crate::parser::{parse_tokens, FunctionScope};
use crate::semantics::{eval_expression};
use crate::value::{Value, StackValue, V};

use crate::expression::Exp;
use crate::expression::Const;
use crate::expression::Var;

use crate::token::Token;
use crate::token::Operand;
use crate::token::Operator;

pub fn run_shell() {
    let mut stack: Vec<StackValue> = Vec::new();

    let main_scope: FunctionScope = FunctionScope {
        input_vars: Vec::new(),
        external_variables: Vec::new(),
        // Current variable scope depth
        var_scope: 0,
        variable_map: HashMap::new()
    };
    let mut function_stack: Vec<FunctionScope> = vec![main_scope];

    let mut rl: Editor<()> = Editor::<()>::new().expect("Error creating editor");
    loop {
        match rl.readline("epilang> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                handle_user_input(line + "\n", &mut stack, &mut function_stack)
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("ReadlineError: {:?}", err);
                break
            }
        };
    }
}

/**
 * Evaluating user input can change the actual scope.
 * The new scope is returned.
 */
fn handle_user_input(
    line: String,
    stack: &mut Vec<StackValue>,
    function_stack: &mut Vec<FunctionScope>
) {
    // Tokenize string
    let mut tokens: Vec<Token> = match tokenize(line) {
        Result::Ok(tokens) => tokens,
        Result::Err(err) => {
            println!("Lexical error: {}", err.msg);
            return
        }
    };
    if tokens.is_empty() { return }

    // We need to handle let expression separately when in interactive mode
    let is_let: bool = match tokens.last() {
        Option::Some(Token::Let) => true,
        _ => false
    };
    if is_let {
        match eval_let(&mut tokens, stack, function_stack) {
            // Increment scope after evaluating let
            Result::Ok(new_scope) => (),
            Result::Err(msg) => println!("{}", msg)
        }
        return
    }
    // Parse tokens to exp
    let exp: Exp = match parse_tokens(&mut tokens, function_stack) {
        Result::Ok(exp) => exp,
        Result::Err(err) => {
            println!("Syntax error: {}", err.msg);
            return
        }
    };
    // Evaluate expression
    match eval_expression(&exp, stack, 0) {
        Result::Ok(V::Ptr(ptr)) => if ptr.is_unit() {} else {println!("{}", ptr.as_ref())},
        Result::Ok(V::Val(value)) => {
            println!("{}", value);
        },
        Result::Err(err) => println!("Error: {}", err.msg)
    }
}

/**
 * We need to handle let expression in a different way when
 * in interactive mode
 */
fn eval_let(
    tokens: &mut Vec<Token>,
    stack: &mut Vec<StackValue>,
    function_stack: &mut Vec<FunctionScope>
) -> Result<(), String> {
    // Pop let token
    match tokens.pop() {
        Option::Some(Token::Let) => (),
        _ => panic!("First token must be a let when calling handle_let function")
    };
    // Pop variable token
    let var_name: String = match tokens.pop() {
        Option::Some(Token::Operand(Operand::Var(name))) => name,
        _ => return Result::Err(String::from("SyntaxError: expected variable token"))
    };

    // Pop "=" token
    match tokens.pop() {
        Option::Some(Token::Operator(Operator::Assign)) => (),
        _ => return Result::Err(String::from("SyntaxError: expected '=' token"))
    };

    let exp: Exp = match parse_tokens(tokens, function_stack) {
        Result::Ok(exp) => exp,
        Result::Err(err) => return Result::Err(String::from(format!("SyntaxError: {}", err.msg)))
    };
    let val = match eval_expression(&exp, stack, 0) {
        Result::Ok(val) => val,
        Result::Err(err) => return Result::Err(err.msg)
    };

    let function_scope: &mut FunctionScope = function_stack.last_mut().unwrap();
    function_scope.variable_map.insert(var_name, function_scope.var_scope);
    function_scope.var_scope += 1;
    match val {
        V::Ptr(ptr) => stack.push(ptr),
        V::Val(value) => stack.push(StackValue::from_box(Box::new(value)))
    }
    Result::Ok(())
}
