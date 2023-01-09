use std::collections::HashMap;

use crate::expression::Exp;
use crate::expression::Const;
use crate::expression::Var;

use crate::token::Token;
use crate::token::Operand;
use crate::token::Operator;

pub struct SyntaxError {
    pub msg: String
}

pub struct FunctionScope {
    pub input_vars: Vec<Var>,
    pub external_variables: Vec<Var>,
    pub var_scope: usize,
    pub variable_map: HashMap<String, usize>
}

pub fn parse(tokens: &mut Vec<Token>) -> Result<Exp, SyntaxError> {
    let main_scope: FunctionScope = FunctionScope {
        input_vars: Vec::new(),
        external_variables: Vec::new(),
        // Current variable scope depth
        var_scope: 0,
        variable_map: HashMap::new()
    };
    parse_tokens(tokens, &mut vec![main_scope])
}

/**
 * If we are inside some function declaration, then this function input variables are located at `function_input_vars.last()`
 */
pub fn parse_tokens(tokens: &mut Vec<Token>, function_stack: &mut Vec<FunctionScope>) -> Result<Exp, SyntaxError> {
    // Initialize the operator stack
    let mut stack: Vec<Token> = Vec::new();
    // Initialize the output queue
    let mut out: Vec<Exp> = Vec::new();

    // When callable is true the `(` token is interpreted as a function call
    let mut callable = false;
    // If we are parsing some function call arguments, then the round bracket that marks the beginning of this function call is
    // in the operator stack at position `call_scope.last()`
    let mut call_scope: Vec<usize> = Vec::new();

    loop {
        if tokens.is_empty() { break };
        let token = tokens.pop().unwrap();
        match &token {
            // Push variable to out. Error if not present in scope
            Token::Operand(Operand::Var(x)) => match function_stack.last().unwrap().variable_map.get(x) {
                Option::Some(scope) => out.push(Exp::Var(Var{name: x.clone(), scope: *scope})),
                Option::None => return Result::Err(SyntaxError{msg: String::from(format!("Unknow variable {}", x))})
            }
            // Operands that are not variables are easily pushed to out
            Token::Operand(o) => out.push(o.to_exp()),

            // Handle operator tokens. Remember that also `;` is an operator, like also `+`, `=`, etc...
            Token::Operator(op) => {
                match op {
                    Operator::Seq => match tokens.last() {
                        // If `;` is the last token in a scope we automatically add `unit` at the end
                        Option::Some(Token::CurlyBracketClosed) => tokens.push(Token::Operand(Operand::Null)),
                        Option::None => tokens.push(Token::Operand(Operand::Null)),
                        // Reduce sequences of multiple subsequent `;` to only one element
                        Option::Some(Token::Operator(Operator::Seq)) => continue,
                        _ => ()
                    },
                    _ => ()
                }
                handle_operator_token(*op, &mut stack, &mut out)?
            },

            Token::RoundBracketOpen => handle_rould_bracket_open_token(tokens, &mut stack, &mut out, &mut call_scope, callable)?,

            Token::RoundBracketClosed => handle_round_bracket_closed_token(tokens, &mut stack, &mut out, &mut call_scope, true)?,

            Token::If => stack.push(Token::If),

            Token::Else => stack.push(Token::Else),

            Token::Let => {
                // Declaring a variable increments the variable scope
                handle_let_token(tokens, &mut stack, function_stack.last_mut().unwrap())?
            },

            Token::Fn => {
                let mut variable_map: HashMap<String, usize> = HashMap::new();
                let input_var_names: Vec<String> = parse_function_def(tokens)?;
                let mut input_vars: Vec<Var> = Vec::with_capacity(input_var_names.len());
                for i in 0..input_var_names.len() {
                    variable_map.insert(input_var_names[i].clone(), i);
                    input_vars.push(Var{name: input_var_names[i].clone(), scope: i});
                }
                function_stack.push(FunctionScope {
                    input_vars: input_vars,
                    external_variables: Vec::new(),
                    var_scope: 0,
                    variable_map: variable_map
                });
                stack.push(Token::Fn)
            },

            Token::CurlyBracketOpen => stack.push(Token::CurlyBracketOpen),

            Token::CurlyBracketClosed => {
                // Closing curly brackets can decrement scope
                handle_curly_bracket_closed_token(&mut stack, &mut out, function_stack)?;
                // After closing a curly bracket we automatically insert `;` if not present.
                // This makes the syntax more similar to Java, C++ etc
                match tokens.last() {
                    Option::Some(Token::Operator(Operator::Seq)) => (),
                    Option::Some(Token::Else) => (),
                    Option::Some(Token::CurlyBracketClosed) => (),
                    Option::None => (),
                    _ => tokens.push(Token::Operator(Operator::Seq))
                }
            },
            Token::Comma => stack.push(Token::Comma),
        }
        // Updates the callable state after processing the token
        callable = token.is_callable();
    }

    loop {
        match stack.pop() {
            Option::None => break,
            Option::Some(Token::RoundBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected character (")}),
            Option::Some(Token::Operator(op)) => {
                let result: Result<(), SyntaxError> = push_operator_to_out(&op, &mut out);
                match result {
                    Result::Ok(()) => (),
                    Result::Err(err) => return Result::Err(err)
                };
            }
            Option::Some(Token::Let) => {
                function_stack.last_mut().unwrap().var_scope -= 1;
                push_let_expr_to_out(&mut out, function_stack.last_mut().unwrap().var_scope)?
            }
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token")}),
            Option::Some(Token::CurlyBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected `{`")}),
            Option::Some(Token::CurlyBracketClosed) => return Result::Err(SyntaxError{msg: String::from("Unexpected `}`")}),
            Option::Some(Token::If) => return Result::Err(SyntaxError{msg: String::from("Unexpected `if`")}),
            Option::Some(Token::Else) => return Result::Err(SyntaxError{msg: String::from("Unexpected `else`")}),
            Option::Some(Token::Comma) => return Result::Err(SyntaxError{msg: String::from("Unexpected `,`")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
        }
    }
    
    if out.len() != 1 {
        Result::Err(SyntaxError{msg: String::from("Can not parse a single expression. Probabily missing a ;")})
    } else if !stack.is_empty() {
        Result::Err(SyntaxError{msg: String::from("Can not parse a single expression. Probabily missing a ;")})
    } else {
        Result::Ok(out.pop().unwrap())
    }
}

fn handle_let_token(
    tokens: &mut Vec<Token>,
    stack: &mut Vec<Token>,
    function_scope: &mut FunctionScope,
) -> Result<(), SyntaxError> {

    match tokens.last() {
        Option::Some(Token::Operand(Operand::Var(name))) => {
            function_scope.variable_map.insert(name.clone(), function_scope.var_scope);
        },
        _ => return Result::Err(SyntaxError{msg: String::from("Expected variable name after let")})
    };

    stack.push(Token::Let);
    function_scope.var_scope += 1;
    Result::Ok(())
}

fn handle_rould_bracket_open_token(tokens: &mut Vec<Token>, stack: &mut Vec<Token>, out: &mut Vec<Exp>,  call_scope: &mut Vec<usize>, callable: bool) -> Result<(), SyntaxError> {
    if callable {
        let stack_index =  stack.len();
        call_scope.push(stack_index);
        stack.push(Token::RoundBracketOpen);
        match tokens.last() {
            Option::Some(Token::RoundBracketClosed) => {
                // This is a function call with zero arguments. We need to call handle_round_bracket_closed_token with `args = false`
                tokens.pop();
                handle_round_bracket_closed_token(tokens, stack, out, call_scope, false)?;
            }
            _ => ()
        }
    } else {
        stack.push(Token::RoundBracketOpen)
    }
    Result::Ok(())
}

/**
 * This function is called when a CurlyBracketClosed Token is found.
 * While popping elements from the operator stack, we decrement the scope by 1 every time we find
 * a Let token. The new scope value is returned.
 */
fn handle_curly_bracket_closed_token(stack: &mut Vec<Token>, out: &mut Vec<Exp>, function_stack: &mut Vec<FunctionScope>) -> Result<(), SyntaxError> {
    loop {
        match stack.pop() {
            Option::Some(Token::CurlyBracketOpen) => break,
            Option::Some(Token::Operator(op)) => {
                let result: Result<(), SyntaxError> = push_operator_to_out(&op, out);
                match result {
                    Result::Ok(()) => (),
                    Result::Err(err) => return Result::Err(err)
                };
            },
            // If we find a let token we decrement the scope and build the let expression
            Option::Some(Token::Let) => {
                function_stack.last_mut().unwrap().var_scope -= 1;
                push_let_expr_to_out(out, function_stack.last().unwrap().var_scope)?
            },
            Option::Some(Token::Comma) => return Result::Err(SyntaxError{msg: String::from("Unexpected `,`")}),
            Option::None => return Result::Err(SyntaxError{msg: String::from("Curly brackets mismatch")}),
            Option::Some(Token::RoundBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token in curly brackets")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
            Option::Some(Token::If) => panic!("Found If in parser operator stack"),
            Option::Some(Token::Else) => panic!("Found Else in parser operator stack")
        }
    };
    match stack.last() {
        // Check if this curly bracket closes an if scope
        Option::Some(Token::If) => {
            stack.pop();
            if out.len() < 2 { return Result::Err(SyntaxError{msg: String::from("Malformed if")}) }
            let if_branch: Exp = out.pop().unwrap();
            let if_clause: Exp = out.pop().unwrap();
            out.push(Exp::IfThenElse(Box::new(if_clause), Box::new(if_branch), Box::new(Exp::Const(Const::None))))
        },
        // Check if this curly bracket closes an else scope
        Option::Some(Token::Else) => {
            stack.pop();
            if out.len() < 2 { return Result::Err(SyntaxError{msg: String::from("Malformed else")}) }
            let else_branch: Exp = out.pop().unwrap();
            match out.pop() {
                Option::Some(Exp::IfThenElse(clause, if_branch, none_branch)) => {
                    match *none_branch {
                        Exp::Const(Const::None) => {
                            out.push(Exp::IfThenElse(clause, if_branch, Box::new(else_branch)))
                        },
                        _ => return Result::Err(SyntaxError{msg: String::from("If expression already has an else branch")})
                    }
                },
                _ => return Result::Err(SyntaxError{msg: String::from("Unexpected else")})
            }
        },
        // Check if this curly bracket closes a function declaration
        Option::Some(Token::Fn) => {
            stack.pop();
            let function: FunctionScope = function_stack.pop().ok_or(SyntaxError{msg: String::from("WTF is going on? I thought this could never happen xD")})?;
            let body: Exp = out.pop().ok_or(SyntaxError{msg: String::from("Missing function declaration body")})?;
            let mut args: Vec<Var> = Vec::new();
            for arg in function.input_vars {
                args.push(arg)
            }
            out.push(Exp::Function(args, Box::new(body)))
        }
        _ => ()
    };
    // Return new scope value
    Result::Ok(())
}

/**
 * When we find a RoundBracketClosed Token, then we pop all the tokens from the operator stack until we find
 * a RoundBracketOpen. If the RoundBracketOpen position in the operator stack matches the one in the call scope,
 * then this is a function call, otherwise these are just a regular grouping brackets.
 */
fn handle_round_bracket_closed_token(tokens: &mut Vec<Token>, stack: &mut Vec<Token>, out: &mut Vec<Exp>, call_scope: &mut Vec<usize>, args: bool) -> Result<(), SyntaxError> {
    let is_function_call: bool;
    // Case of functions and function calls with zero arguments is a special case covered when an open round bracket is found
    let mut num_arguments: usize = if args { 1 } else { 0 };
    loop {
        match stack.pop() {
            Option::None => return Result::Err(SyntaxError{msg: String::from("Mismatched round brackets")}),
            Option::Some(Token::RoundBracketOpen) => {
                // Check if this is a function call or not
                is_function_call = call_scope.last()
                    .map(|stack_index: &usize| { *stack_index == stack.len()})
                    .unwrap_or(false);
                break
            },
            Option::Some(Token::Operator(op)) => push_operator_to_out(&op, out)?,
            Option::Some(Token::Comma) => num_arguments += 1,
            Option::Some(Token::Let) => return Result::Err(SyntaxError{msg: String::from("Unexpected let statement in round brackets")}),
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token in round brackets")}),
            Option::Some(Token::CurlyBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
            Option::Some(Token::If) => panic!("Found If in parser operator stack"),
            Option::Some(Token::Else) => panic!("Found Else in parser operator stack")
        }
    };
    if is_function_call {
        // Decrement the function call scope by one
        call_scope.pop();
        // Get function call arguments from autput queue
        let mut args: Vec<Exp> = Vec::with_capacity(num_arguments);
        for _ in 0..num_arguments {
            let arg: Exp = out.pop().ok_or(SyntaxError{msg: String::from("Wrong number of function call arguments")})?;
            args.push(arg)
        }
        args.reverse();
        // Build function call expression
        let callable_exp: Exp = out.pop().ok_or(SyntaxError{msg: String::from("Missing callable expression before function call")})?;
        out.push(Exp::FunctionCall(Box::new(callable_exp), args));
    }
    else if num_arguments > 1 {
        return  Result::Err(SyntaxError{msg: String::from("Unexpected `,` inside round brackets")})
    }
    Result::Ok(())
}

fn handle_operator_token(op: Operator, stack: &mut Vec<Token>, out: &mut Vec<Exp>) -> Result<(), SyntaxError> {
    loop {
        match stack.last() {
            Option::None => break,
            Option::Some(
                Token::RoundBracketOpen |
                Token::CurlyBracketOpen |
                Token::If |
                Token::Else |
                Token::Let |
                Token::Fn |
                Token::Comma
            ) => break,
            Option::Some(Token::Operator(o2)) => {
                if o2.precedence() >= op.precedence() {
                    break
                } else {
                    match push_operator_to_out(o2, out) {
                        Result::Ok(()) => stack.pop(),
                        Result::Err(err) => return Result::Err(err)
                    };
                }
            },
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_) ) => panic!("Found Operand in parser operator stack"),
        }
    }
    stack.push(Token::Operator(op));
    Result::Ok(())
}

fn parse_function_def(tokens: &mut Vec<Token>) -> Result<Vec<String>, SyntaxError> {
    match tokens.pop() {
        Option::Some(Token::RoundBracketOpen) => (),
        _ => return Result::Err(SyntaxError{msg: String::from("")})
    };
    let mut input_vars: Vec<String> = Vec::new();
    loop {
        match tokens.pop() {
            Option::Some(Token::RoundBracketClosed) => break,
            Option::Some(Token::Operand(Operand::Var(name))) => {
                input_vars.push(name);
                match tokens.pop() {
                    Option::Some(Token::RoundBracketClosed) => break,
                    Option::Some(Token::Comma) => (),
                    _ => return Result::Err(SyntaxError{msg: String::from("Expected `,` after function argument")})
                }
            },
            _ => return Result::Err(SyntaxError{msg: String::from("Malformed function params")})
        };
    };
    Result::Ok(input_vars)
}

fn push_let_expr_to_out(out: &mut Vec<Exp>, scope: usize) -> Result<(), SyntaxError> {
    let (exp1, exp2) : (Exp, Box<Exp>) = match out.pop() {
        // If the next element in the queue is not `;` return error
        Option::Some(Exp::Seq(exp1, exp2)) => (*exp1, exp2),
        _ => return Result::Err(SyntaxError { msg:  String::from("Expected ; after let")})
    };
    match exp1 {
        // Case when variable is assigned during declaration
        Exp::Assign(var, right_exp) => {
            if var.scope != scope {
                return Result::Err(SyntaxError{
                    msg: String::from(format!("Variable {} has scope {}, but was expecting scope {}", var.name, var.scope, scope))
                })
            }
            out.push(Exp::Decl(var, right_exp, exp2));
        },
        // Case when variable is declared but not assigned
        Exp::Var(var) => {
            if var.scope != scope {
                return Result::Err(SyntaxError{
                    msg: String::from(format!("Variable {} has scope {}, but was expecting scope {}", var.name, var.scope, scope))
                })
            }
            let none = Box::new(Exp::Const(Const::None));
            out.push(Exp::Decl(var, none, exp2));
        },
        _ => return Result::Err(SyntaxError{msg: String::from("Expecting variable or assignment after let")})
    };
    Result::Ok(())
}

fn push_operator_to_out(op: &Operator, out: &mut Vec<Exp>) -> Result<(), SyntaxError> {
    match op {
        Operator::Seq => {
            if out.is_empty() { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let exp2: Exp = out.pop().unwrap();
            let exp1: Exp = out.pop().unwrap_or(Exp::Const(Const::None));
            out.push(Exp::Seq(Box::new(exp1), Box::new(exp2)))
        },
        Operator::Assign => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (exp, var) = (out.pop().unwrap(), out.pop().unwrap());
            match var {
                Exp::Var(var) => out.push(Exp::Assign(var, Box::new(exp))),
                _ => return Result::Err(SyntaxError{msg: String::from("Expected variable before assignment")})
            }
        },
        Operator::Mul => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Mul(Box::new(o1), Box::new(o2)))
        },
        Operator::Div => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Div(Box::new(o1), Box::new(o2)))
        },
        Operator::Sum => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Sum(Box::new(o1), Box::new(o2)))
        },
        Operator::Sub => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Sub(Box::new(o1), Box::new(o2)))
        },
        Operator::Lt => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Lt(Box::new(o1), Box::new(o2)))
        },
        Operator::Gt => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Gt(Box::new(o1), Box::new(o2)))
        },
        Operator::Eq => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Eq(Box::new(o1), Box::new(o2)))
        },
        Operator::Neq => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Neq(Box::new(o1), Box::new(o2)))
        },
        Operator::And => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::And(Box::new(o1), Box::new(o2)))
        },
        Operator::Or => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Or(Box::new(o1), Box::new(o2)))
        },
        Operator::Not => {
            if out.len() < 1 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let o = out.pop().unwrap();
            out.push(Exp::Not(Box::new(o)))
        },
    }
    Result::Ok(())
}
