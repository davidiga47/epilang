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

    loop {
        if tokens.is_empty() { break };
        let token = tokens.pop().unwrap();
        match &token {
            // Push variable to out. Error if not present in scope
            Token::Operand(Operand::Var(x)) => match function_stack.last().unwrap().variable_map.get(x) {
                Option::Some(scope) => out.push(Exp::Var(Var{name: x.clone(), scope: *scope})),
                Option::None => return Result::Err(SyntaxError{msg: String::from(format!("Unknown variable {}", x))})
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
                handle_operator_token(*op, &mut stack, &mut out, tokens)?
            },

            Token::ListSelectionOpen => stack.push(Token::ListSelectionOpen),

            Token::FunctionCallOpen => {
                stack.push(Token::FunctionCallOpen);
                // If This is a function call with zero arguments, then we need to call handle_round_bracket_closed_token with `args = false`
                match tokens.last() {
                    Option::Some(Token::RoundBracketClosed) => {
                        tokens.pop();
                        handle_round_bracket_closed_token(tokens, &mut stack, &mut out, false)?;
                    }
                    _ => ()
                }
            }

            Token::RoundBracketOpen => stack.push(Token::RoundBracketOpen),

            Token::RoundBracketClosed => handle_round_bracket_closed_token(tokens, &mut stack, &mut out, true)?,

            Token::SquareBracketOpen => stack.push(Token::SquareBracketOpen),

            Token::SquareBracketClosed => handle_square_bracket_closed_token(&mut stack, &mut out, false)?,

            Token::While => stack.push(Token::While),

            Token::If => stack.push(Token::If),

            Token::Else => stack.push(Token::Else),

            Token::Try => stack.push(Token::Try),
            
            Token::Catch => {
                stack.push(Token::Catch);
                let function_scope=function_stack.last_mut().unwrap();
                //searches for an exception to catch, it needs to add it to the variables environment 
                //otherwise the interpreter will try to evaluate the exception and fail
                match tokens.last() {
                    Option::Some(Token::Operand(Operand::Var(name))) => {
                        function_scope.variable_map.insert(name.clone(), function_scope.var_scope);
                    },
                    _ =>return Result::Err(SyntaxError{msg: String::from("Expected variable name after catch token")})
                };
            },

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

            Token::CurlyBracketOpen => {
                match stack.last(){
                    Option::Some(Token::In) => {
                        tokens.push(Token::Operator(Operator::Seq));
                        tokens.push(Token::Operand(Operand::Null));
                        tokens.push(Token::Operator(Operator::Assign));
                        let cont=match out.last(){
                            Option::Some(Exp::Var(v)) => &v.name,
                            _ => return Result::Err(SyntaxError{msg: String::from("Unknown error")})
                        };
                        tokens.push(Token::Operand(Operand::Var(cont.clone())));
                        tokens.push(Token::Let);
                    },
                    _ => ()
                };
                stack.push(Token::CurlyBracketOpen)
            },

            Token::CurlyBracketClosed => {
                // Closing curly brackets can decrement scope
                handle_curly_bracket_closed_token(&mut stack, &mut out, function_stack)?;
                // After closing a curly bracket we automatically insert `;` if not present.
                // This makes the syntax more similar to Java, C++ etc
                match tokens.last() {
                    Option::Some(Token::Operator(Operator::Seq)) => (),
                    Option::Some(Token::Catch) => (),
                    Option::Some(Token::Else) => (),
                    Option::Some(Token::CurlyBracketClosed) => (),
                    Option::None => (),
                    _ => tokens.push(Token::Operator(Operator::Seq))
                }

            },
            Token::Comma => {
                loop {
                    match stack.last() {
                        Option::Some(
                            Token::SquareBracketOpen |
                            Token::FunctionCallOpen |
                            Token::Comma
                        ) => break,
                        Option::Some(Token::Operator(op)) => {
                            push_operator_to_out(op, &mut out)?;
                            stack.pop();
                        },
                        Option::Some(tkn) => return Result::Err(SyntaxError{msg: format!("Unexpected token {}", tkn)}),
                        Option::None => return Result::Err(SyntaxError{msg: String::from("Unexpected token `,`",)})
                    }
                }
                stack.push(Token::Comma)
            },

            Token::Callcc => {
                stack.push(Token::Callcc);
                //we need to add the continuation label to the variables environment, for the same reason as above with catch
                let function_scope=function_stack.last_mut().unwrap();
                match tokens.last() {
                    Option::Some(Token::Operand(Operand::Var(name))) => {
                        function_scope.variable_map.insert(name.clone(), function_scope.var_scope);
                    },
                    _ =>return Result::Err(SyntaxError{msg: String::from("Expected variable name after callcc token")})
                };
            },

            Token::In => {
                match stack.last(){
                    Option::Some(Token::Callcc) => stack.push(Token::In),
                    _ => return Result::Err(SyntaxError{msg: format!("Unexpected token `in`")})
                }
            }
        }
    }

    loop {
        match stack.pop() {
            Option::None => break,
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
            Option::Some(Token::RoundBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected bracket `(`")}),
            Option::Some(Token::FunctionCallOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected function call open `(`")}),
            Option::Some(Token::ListSelectionOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected list selection open `[`")}),
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token")}),
            Option::Some(Token::CurlyBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected `{`")}),
            Option::Some(Token::SquareBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Unexpected `[`")}),
            Option::Some(Token::CurlyBracketClosed) => return Result::Err(SyntaxError{msg: String::from("Unexpected `}`")}),
            Option::Some(Token::While) => return Result::Err(SyntaxError{msg: String::from("Unexpected `while`")}),
            Option::Some(Token::If) => return Result::Err(SyntaxError{msg: String::from("Unexpected `if`")}),
            Option::Some(Token::Else) => return Result::Err(SyntaxError{msg: String::from("Unexpected `else`")}),
            Option::Some(Token::Try) => return Result::Err(SyntaxError{msg: String::from("Unexpected `try`")}),
            Option::Some(Token::Catch) => return Result::Err(SyntaxError{msg: String::from("Unexpected `catch`")}),
            Option::Some(Token::Comma) => return Result::Err(SyntaxError{msg: String::from("Unexpected `,`")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::SquareBracketClosed) => panic!("Found SquareBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
            Option::Some(Token::Callcc) => panic!("Found Callcc in parser operator stack"),
            Option::Some(Token::In) => panic!("Found 'In' token in parser operator stack")
        }
    }
    
    if out.len() != 1  {
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
        _ =>return Result::Err(SyntaxError{msg: String::from("Expected variable name after let")})
    };

    stack.push(Token::Let);
    function_scope.var_scope += 1;
    Result::Ok(())
}

fn handle_square_bracket_closed_token(stack: &mut Vec<Token>, out: &mut Vec<Exp>, empty: bool) -> Result<(), SyntaxError> {
    let is_selection: bool;
    let mut len: usize = if empty { 0 } else { 1 };

    loop {
        match stack.pop() {
            Option::Some(Token::SquareBracketOpen) => {
                is_selection = false;
                break
            },
            Option::Some(Token::ListSelectionOpen) => {
                is_selection = true;
                break
            }
            Option::Some(Token::Operator(op)) => push_operator_to_out(&op, out)?,
            Option::Some(Token::Comma) => len += 1,
            Option::Some(Token::Let) => return Result::Err(SyntaxError{msg: String::from("Unexpected let statement in round brackets")}),
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token in round brackets")}),
            Option::Some(Token::FunctionCallOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::RoundBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::CurlyBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Curly brackets mismatch")}),
            Option::None => return Result::Err(SyntaxError{msg: String::from("Mismatched round brackets")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::SquareBracketClosed) => panic!("Found SquareBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
            Option::Some(Token::While) => panic!("Found While in parser operator stack"),
            Option::Some(Token::If) => panic!("Found If in parser operator stack"),
            Option::Some(Token::Else) => panic!("Found Else in parser operator stack"),
            Option::Some(Token::Try) => panic!("Found try in parser operator stack"),
            Option::Some(Token::Catch) => panic!("Found catch in parser operator stack"),
            Option::Some(Token::Callcc) => panic!("Found callcc in parser operator stack"),
            Option::Some(Token::In) => panic!("Found in in parser operator stack")
        }
    };
    if is_selection {
        if len != 1 {
            return Result::Err(SyntaxError{msg: String::from("Unexpected `,` in list selection")})
        }
        // Get index from output queue
        let index: Exp = out.pop().ok_or(SyntaxError{msg: String::from("List selection must contain one expression")})?;
        // Build list selection expression
        let list: Exp = out.pop().ok_or(SyntaxError{msg: String::from("Missing list expression before list selection")})?;
        out.push(Exp::ListSelection(Box::new(list), Box::new(index)));
    } else {
        // Get list elements
        let mut list: Vec<Exp> = Vec::with_capacity(len);
        for _ in 0..len {
            let elem: Exp = out.pop().ok_or(SyntaxError{msg: String::from("Malformed list")})?;
            list.push(elem)
        }
        list.reverse();
        out.push(Exp::List(list));
    };
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
            Option::Some(Token::FunctionCallOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::RoundBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::ListSelectionOpen) => return Result::Err(SyntaxError{msg: String::from("Square brackets mismatch")}),
            Option::Some(Token::SquareBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Square brackets mismatch")}),
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token in curly brackets")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::SquareBracketClosed) => return Result::Err(SyntaxError{msg: String::from("Found SquareBracketClosed in parser operator stack")}),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
            Option::Some(Token::While) => panic!("Found While in parser operator stack"),
            Option::Some(Token::If) => panic!("Found If in parser operator stack"),
            Option::Some(Token::Else) => panic!("Found Else in parser operator stack"),
            Option::Some(Token::Try) => panic!("Found try in parser operator stack"),
            Option::Some(Token::Catch) => panic!("Found catch in parser operator stack"),
            Option::Some(Token::Callcc) => panic!("Found callcc in parser operator stack"),
            Option::Some(Token::In) => panic!("Found in in parser operator stack")
        }
    };
    match stack.last() {
        // Check if this curly bracket closes a while scope
        Option::Some(Token::While) => {
            stack.pop();
            if out.len() < 2 { return Result::Err(SyntaxError{msg: String::from("Malformed while")}) }
            let while_body: Exp = out.pop().unwrap();
            let guard: Exp = out.pop().unwrap();
            out.push(Exp::While(Box::new(guard), Box::new(while_body)))
        },
        // Check if this curly bracket closes an if scope
        Option::Some(Token::If) => {
            stack.pop();
            if out.len() < 2 { 
                return Result::Err(SyntaxError{msg: String::from("Malformed if")}) 
            }
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
        // Check if this curly bracket closes a Try scope
        Option::Some(Token::Try) => {
            stack.pop();
            if out.len() < 1 { return Result::Err(SyntaxError{msg: String::from("Malformed Try")}) }
            let mut try_block: Exp = out.pop().unwrap();
            match stack.last() {
                Option::Some(Token::Operator(Operator::Throw)) => try_block=Exp::Throw(Box::new(try_block)),
                _ => ()
            };
            out.push(Exp::Try(Box::new(try_block)))
        },
        // Check if this curly bracket closes a Catch scope
        Option::Some(Token::Catch) => {
            stack.pop();
            if out.len() < 2 { return Result::Err(SyntaxError{msg: String::from("Malformed Try-Catch")}) }
            let exc_handler: Exp = out.pop().unwrap();
            let exc_exp: Exp = out.pop().unwrap();  //This is the expression representing the excpetion label (it's a variable for the interpreter)
            let exc: Var;                           
            match exc_exp {
                Exp::Var(v) => exc=v,               //In fact we cast it to a Var
                _ => return Result::Err(SyntaxError{msg: String::from("Malformed Try-Catch")}) 
            }
            match out.pop() {
                Option::Some(Exp::Try(try_block)) => {
                    //if we have previously found a Try then we have a TryCatch expression
                    out.push(Exp::TryCatch(try_block, exc, Box::new(exc_handler)))
                },
                _ => return Result::Err(SyntaxError{msg: String::from("Unexpected Catch")})
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
        },
        Option::Some(Token::In) => {
            //function_stack.last_mut().unwrap().var_scope -= 1;
            stack.pop();
            match stack.last() {
                Option::Some(Token::Callcc) => {
                    stack.pop();
                    if out.len() < 2 { return Result::Err(SyntaxError{msg: String::from("Malformed call/cc")}) };
                    let expr: Exp = out.pop().unwrap();
                    let tmp: Exp = out.pop().unwrap();  //This is the expression representing the continuation label 
                                                        //(it's a variable for the interpreter)
                    let context: Var;
                    match tmp {
                        Exp::Var(v) => context = v,     //In fact we cast it to a Var
                        _ => { return Result::Err(SyntaxError{msg: String::from("Malformed call/cc")}) }
                    };
                    out.push(Exp::Callcc(context.clone(), Box::new(expr)));
                    //push_callcc_expr_to_out(out, function_stack.last().unwrap().var_scope)?
                },
                //If there's not a Callcc token there shouldn't be a In token 
                _ => return Result::Err(SyntaxError{msg: String::from("Malformed call/cc")})
            }
        },
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
fn handle_round_bracket_closed_token(tokens: &mut Vec<Token>, stack: &mut Vec<Token>, out: &mut Vec<Exp>, args: bool) -> Result<(), SyntaxError> {
    let is_function_call: bool;
    // Case of functions and function calls with zero arguments is a special case covered when an open round bracket is found
    let mut num_arguments: usize = if args { 1 } else { 0 };
    loop {
        match stack.pop() {
            Option::None => return Result::Err(SyntaxError{msg: String::from("Mismatched round brackets")}),
            Option::Some(Token::FunctionCallOpen) => {
                is_function_call = true;
                break
            }
            Option::Some(Token::RoundBracketOpen) => {
                is_function_call = false;
                break
            },
            Option::Some(Token::Operator(op)) => push_operator_to_out(&op, out)?,
            Option::Some(Token::Comma) => num_arguments += 1,
            Option::Some(Token::Let) => return Result::Err(SyntaxError{msg: String::from("Unexpected let statement in round brackets")}),
            Option::Some(Token::Fn) => return Result::Err(SyntaxError{msg: String::from("Unexpected `fn` token in round brackets")}),
            Option::Some(Token::SquareBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Square brackets mismatch")}),
            Option::Some(Token::ListSelectionOpen) => return Result::Err(SyntaxError{msg: String::from("Square brackets mismatch")}),
            Option::Some(Token::CurlyBracketOpen) => return Result::Err(SyntaxError{msg: String::from("Round brackets mismatch")}),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::SquareBracketClosed) => panic!("Found SquareBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_)) => panic!("Found Operand in parser operator stack"),
            Option::Some(Token::While) => panic!("Found While in parser operator stack"),
            Option::Some(Token::If) => panic!("Found If in parser operator stack"),
            Option::Some(Token::Else) => panic!("Found Else in parser operator stack"),
            Option::Some(Token::Try) => panic!("Found try in parser operator stack"),
            Option::Some(Token::Catch) => panic!("Found catch in parser operator stack"),
            Option::Some(Token::Callcc) => panic!("Found callcc in parser operator stack"),
            Option::Some(Token::In) => panic!("Found in in parser operator stack")
        }
    };
    if is_function_call {
        // Get function call arguments from output queue
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

fn handle_operator_token(op: Operator, stack: &mut Vec<Token>, out: &mut Vec<Exp>, tokens: &mut Vec<Token>) -> Result<(), SyntaxError> {
    loop {
        match stack.last() {
            Option::None => break,
            Option::Some(
                Token::FunctionCallOpen |
                Token::RoundBracketOpen |
                Token::ListSelectionOpen |
                Token::SquareBracketOpen |
                Token::CurlyBracketOpen |
                Token::While |
                Token::If |
                Token::Else |
                Token::Let |
                Token::Fn |
                Token::Try |
                Token::Catch |
                Token::Comma |
                Token::Callcc |
                Token::In
            ) => break,
            Option::Some(Token::Operator(o2)) => {
                if o2.precedence() >= op.precedence() {
                    break
                } else {
                    push_operator_to_out(o2, out)?;
                    stack.pop();
                }
            },
            Option::Some(Token::CurlyBracketClosed) => panic!("Found CurlyBracketClosed in parser operator stack"),
            Option::Some(Token::SquareBracketClosed) => panic!("Found SquareBracketClosed in parser operator stack"),
            Option::Some(Token::RoundBracketClosed) => panic!("Found RoundBracketClosed in parser operator stack"),
            Option::Some(Token::Operand(_) ) => panic!("Found Operand in parser operator stack"),
        }
    }
    match op {
        Operator::Throw => {
            let tmp=tokens.pop().unwrap();      //we temporarily store the excpetion (or continuation) label
            match tmp {
                Token::Operand(Operand::Var(_)) =>{
                    match tokens.last() {
                        Option::Some(
                            Token::Operator(Operator::Seq) | 
                            Token::RoundBracketClosed |
                            Token::SquareBracketClosed |
                            Token::CurlyBracketClosed
                        ) => stack.push(Token::Operator(Operator::Throw)),
                        Option::None => stack.push(Token::Operator(Operator::Throw)), 
                        //If there's another expression after the label this is a Throwcc expression. Otherwise it's a normal Throw
                        _ => stack.push(Token::Operator(Operator::Throwcc))
                    };
                }
                _ => stack.push(Token::Operator(Operator::Throw))
            };
            tokens.push(tmp);                  //we push again the label we temporarily popped
        },
        _ => stack.push(Token::Operator(op))
    }
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
        Exp::Assign(left_exp, right_exp) => {
            let var = match *left_exp {
                Exp::Var(var) => var,
                _ => return Result::Err(SyntaxError{msg: String::from("Expecting variable name after let")})
            };
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
            out.push(Exp::Seq(Box::new(exp1), Box::new(exp2)));
        },
        Operator::Assign => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (right_exp, left_exp) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Assign(Box::new(left_exp), Box::new(right_exp)))
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
        Operator::Mod => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Mod(Box::new(o1), Box::new(o2)))
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
        Operator::Lte => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Lte(Box::new(o1), Box::new(o2)))
        },
        Operator::Gt => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Gt(Box::new(o1), Box::new(o2)))
        },
        Operator::Gte => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let (o2, o1) = (out.pop().unwrap(), out.pop().unwrap());
            out.push(Exp::Gte(Box::new(o1), Box::new(o2)))
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
        Operator::Throw => {
            if out.len() < 1 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let o = out.pop().unwrap();
            out.push(Exp::Throw(Box::new(o)))            
        },
        Operator::Throwcc => {
            if out.len() < 2 { return Result::Err(SyntaxError{msg: format!("Unexpected operator {}", op)}) }
            let e = out.pop().unwrap();
            let k = out.pop().unwrap();
            match k {
                Exp::Var(v) => out.push(Exp::Throwcc(v,Box::new(e))),
                _ => return Result::Err(SyntaxError{msg: format!("Expected var after callcc token")})   
            }    
        }
    }
    Result::Ok(())
}
    