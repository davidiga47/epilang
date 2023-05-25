use crate::expression::{Exp, Const};
use crate::parser::SyntaxError;
use crate::value::{Value, StackValue, Function, V};
use crate::value::Value::Unit;
use crate::value::V::Val;
use crate::value::V::Ptr;
use substring::Substring;

pub struct Error {
    pub msg: String,
    pub v: V
}

fn exp_to_string(exp: &Exp) -> String {
    match exp {
        Exp::Var(x) => format!("{}", x.name),
        _ => "".to_string()
    }
}

pub fn list_convert(mut l: Vec<StackValue>) -> Vec<Exp> {
    let mut nl=Vec::new();
    loop{
        match l.pop(){
            Option::None => break,
            Option::Some(sv) => match sv.as_ref(){
                Value::Unit => nl.push(Exp::Const(Const::None)),
                Value::Bool(b) => nl.push(Exp::Const(Const::Boolean(*b))),
                Value::Int(i) => {
                    let j: i32=*i as i32;
                    nl.push(Exp::Const(Const::Integer(j)))
                },
                Value::Str(s) => nl.push(Exp::Const(Const::String(s.clone()))),
                Value::List(ol) => nl.push(Exp::List(list_convert(ol.clone()))),
                Value::Fn(f) => nl.push(Exp::Function(Vec::new(),f.body.clone()))
            }
        }
    };
    let mut res_stack=Vec::new();
    loop {
        match nl.pop(){
            Option::None => break,
            Option::Some(x) => res_stack.push(x) 
        }
    };
    return res_stack;
}

pub fn eval(exp: &Exp) -> Result<V, Error> {
    let mut stack: Vec<StackValue> = Vec::new();
    eval_expression(exp, &mut stack, 0, false, "".to_string())
}

pub fn eval_expression(exp: &Exp, stack: &mut Vec<StackValue>, stack_start: usize, in_call: bool, param: String) -> Result<V, Error> {
    match exp {
        Exp::Const(c) => Result::Ok(V::Val(Value::from_const(&c))),

        Exp::Var(x) => {
            Result::Ok(V::Ptr(stack[x.scope + stack_start]))
        },

        Exp::Decl(_, val_exp, exp2) => {
            match eval_expression(val_exp, stack, stack_start, false, "".to_string())? {
                V::Ptr(ptr) => stack.push(ptr),
                V::Val(value) => stack.push(StackValue::from_box(Box::new(value)))
            };
            let result = eval_expression(exp2, stack, stack_start, false, "".to_string());
            stack.pop();
            result
        },

        Exp::List(list) => {
            let mut values: Vec<StackValue> = Vec::with_capacity(list.len());
            for exp in list {
                let value = match eval_expression(exp, stack, stack_start, false, "".to_string())? {
                    V::Ptr(ptr) => ptr,
                    V::Val(value) => StackValue::from_box(Box::new(value))
                };
                values.push(value)
            }
            Result::Ok(V::Val(Value::List(values)))
        }

        Exp::ListSelection(list, index) => {
            let list: V = eval_expression(list, stack, stack_start, false, "".to_string())?;
            let index: V = eval_expression(index, stack, stack_start, false, "".to_string())?;
            let value = match (list.as_ref(), index.as_ref()) {
                (Value::List(values), Value::Int(i)) => values.get(*i as usize)
                    .ok_or(Error{msg: String::from("List index out of range"), v:Val(Unit)})?,
                _ => return Result::Err(Error{msg: String::from("q"), v:Val(Unit)})
            };
            Result::Ok(V::Ptr(*value))
        }

        Exp::Assign(left_exp, right_exp) => {
            let right_value: V = eval_expression(right_exp, stack, stack_start, false, "".to_string())?;
            match (*left_exp).as_ref() {
                Exp::Var(var) => match right_value {
                    V::Ptr(ptr) => stack[var.scope + stack_start] = ptr,
                    V::Val(value) => stack[var.scope + stack_start] = StackValue::from_box(Box::new(value))
                },
                Exp::ListSelection(list, index) => {
                    let mut list = eval_expression(list.as_ref(), stack, stack_start, false, "".to_string())?;
                    let list: &mut Vec<StackValue> = match list.as_mut_ref() {
                        Value::List(list) => list,
                        _ => return Result::Err(Error{msg: String::from("Expected list value before list selection"), v:Val(Unit)})
                    };
                    let index: usize = match eval_expression(index.as_ref(), stack, stack_start, false, "".to_string())?.as_ref() {
                        Value::Int(i) => *i as usize,
                        _ => return Result::Err(Error{msg: String::from("Expected number in list selection"), v:Val(Unit)})
                    };
                    if index >= list.len() {
                        return Result::Err(Error{msg: String::from("List index out of range"), v:Val(Unit)})
                    }
                    list[index] = match right_value {
                        V::Ptr(ptr) => ptr,
                        V::Val(value) => StackValue::from_box(Box::new(value))
                    }
                },
                _ => return Result::Err(Error{msg: String::from("Invalid left-hand side in assignment"), v:Val(Unit)})
            }
            Result::Ok(V::Ptr(StackValue::unit()))
        }

        Exp::While(guard, exp) => {
            let condition = eval_expression(guard, stack, stack_start, false, "".to_string())?.as_bool();
            if !condition {
                Result::Ok(V::Val(Value::Unit))
            } else {
                loop {
                    let v: V = eval_expression(exp, stack, stack_start, false, "".to_string())?;
                    if !eval_expression(guard, stack, stack_start, false, "".to_string())?.as_bool() {
                        break Result::Ok(v)
                    }
                }
            }
        }

        Exp::IfThenElse(condition, exp1, exp2) => {
            let is_true: bool = eval_expression(condition, stack, stack_start, false, "".to_string())?.as_bool();
            eval_expression(if is_true {exp1} else {exp2}, stack, stack_start, false, "".to_string())
        }

        Exp::Try(exp1)=> {
            let res: Result<V, Error> = eval_expression(exp1,stack,stack_start, false, "".to_string());
            match res {
                Result::Ok(_) => return Result::Ok(res?),
                Result::Err(Error{msg, v}) => {
                    return Result::Err(Error{msg, v})
                }
            }
        }
        
        Exp::TryCatch(exp1,exc,exp2) => {
            let res: Result<V, Error> = eval_expression(exp1,stack,stack_start, false, "".to_string());
            match res {
                Result::Ok(_) => return Result::Ok(res?),
                Result::Err(Error{msg, v}) => {
                    let val_exp = match v.as_ref() {
                        Value::Unit => Exp::Const(Const::None),
                        Value::Int(x) => Exp::Const(Const::Integer((*x).try_into().unwrap())),
                        Value::Bool(b) => Exp::Const(Const::Boolean(*b)),
                        Value::Fn(f) => Exp::Function(Vec::new(), f.body.clone()),
                        Value::Str(s) => Exp::Const(Const::String(s.to_string())),
                        Value::List(l) => Exp::List(list_convert(l.clone()))
                    };
                    let tmp: Result<V, Error> = eval_expression(&Exp::Decl(exc.clone(),Box::new(val_exp),exp2.clone()),stack,stack_start, false, "".to_string());
                    match tmp {
                        Result::Ok(v) => {
                            return Result::Ok(v)
                        }
                        Result::Err(Error{msg, v}) => return Result::Err(Error{msg, v}), 
                        _ => return Result::Err(Error{msg: String::from("Unknown error"), v:Val(Unit)})
                    }
                }
            }
        }

        //evaluate the exception then returns the string containing the value. Try-Catch, if present, will handle the exception thrown
        Exp::Throw(exp) => {
            let res=eval_expression(exp,stack,stack_start, false, "".to_string())?;
            return Result::Err(Error{msg: String::from("uncaught exception ".to_string()+&res.to_string()), v:res})
        },

        //E.g. throw k 2 => this is used to evaluate a block of the type `callcc k in e`
        Exp::Throwcc(k,e) => {
            let res=eval_expression(e,stack,stack_start, false, "".to_string())?;
            if in_call {return Result::Err(Error{msg: String::from(param), v:res})};
            return Result::Err(Error{msg: String::from(k.name.to_string()), v:res})
        },

        //calls the current continuation as k and then evaluates the expression e. 
        //If k is thrown inside e with `throw k m`, then `callcc k in e` evaluates to m
        Exp::Callcc(k,e) => {
            let res: Result<V, Error> = eval_expression(e,stack,stack_start, false, "".to_string());
            match res {
                Result::Ok(_) => return Result::Ok(res?),
                Result::Err(Error{msg, v}) => {
                    if (msg.len()>18 && msg.substring(0,18).eq(&"uncaught exception".to_string())) { return Result::Err(Error{msg: msg, v: v})};
                    if (msg.eq(&k.name.to_string())) {return Result::Ok(v)};
                    return eval_expression(e,stack,stack_start, false, "".to_string())
                }
                
            }
        },

        Exp::Function(args, body) => {
            Result::Ok(V::Val(Value::Fn(Function { num_args: args.len(), external_values: Vec::new(), body: body.clone() })))
        },

        Exp::FunctionCall(callable, args) => {
            match eval_expression(callable, stack, stack_start, false, "".to_string())?.as_ref() {
                Value::Fn(function) => {
                    if args.len() != function.num_args {
                        return Result::Err(Error { msg: format!("Wrong number of arguments. Expected {}, found {}", function.num_args, args.len()), v:Val(Unit) })
                    }
                    let function_stack_start: usize = stack.len();
                    for arg in args {
                        match eval_expression(arg, stack, stack_start, false, "".to_string())? {
                            V::Ptr(ptr) => stack.push(ptr),
                            V::Val(value) => stack.push(StackValue::from_box(Box::new(value)))
                        };
                    };
                    let result: V = eval_expression(function.body.as_ref(), stack, function_stack_start, true, exp_to_string(&args[0]))?;
                    for _ in 0..function.num_args {
                        stack.pop();
                    }
                    Result::Ok(result)
                },
                _ => return Result::Err(Error{msg: String::from("Expression is not callable"), v:Val(Unit)})
            }
        },

        Exp::Seq(exp1, exp2) => {
            eval_expression(exp1, stack, stack_start, false, "".to_string())?;
            eval_expression(exp2, stack, stack_start, false, "".to_string())
        },

        Exp::Sum(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(sum(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Sub(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(sub(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Mul(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(mul(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Mod(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(modulo(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Div(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(div(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Lt(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(lt(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Lte(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(lte(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Gt(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(gt(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Gte(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(gte(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Eq(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(eq(val1.as_ref(), val2.as_ref())?))
        },

        Exp::Neq(exp1, exp2) => {
            let (val1, val2) = double_eval(exp1, exp2, stack, stack_start)?;
            Result::Ok(V::Val(neq(val1.as_ref(), val2.as_ref())?))
        },

        Exp::And(exp1, exp2) => {
            let val_exp1 = eval_expression(exp1, stack, stack_start, false, "".to_string())?;
            if (!val_exp1.as_bool()) {
                Result::Ok(V::Val(Value::Bool(false)))
            }
            else {
                let val_exp2 = eval_expression(exp2, stack, stack_start, false, "".to_string())?;
                Result::Ok(V::Val(Value::Bool(val_exp2.as_bool())))
            }
        },

        Exp::Or(exp1, exp2) => {
            let val_exp1 = eval_expression(exp1, stack, stack_start, false, "".to_string())?;
            if (val_exp1.as_bool()) {
                Result::Ok(V::Val(Value::Bool(true)))
            }
            else {
                let val_exp2 = eval_expression(exp2, stack, stack_start, false, "".to_string())?;
                Result::Ok(V::Val(Value::Bool(val_exp2.as_bool())))
            }
        },

        Exp::Not(exp1) => {
            let v = eval_expression(exp1, stack, stack_start, false, "".to_string())?;
            Result::Ok(V::Val(Value::Bool(!v.as_bool())))
        }
    }
}

fn double_eval(exp1: &Exp, exp2: &Exp, stack: &mut Vec<StackValue>, stack_start: usize) -> Result<(V, V), Error> {
    let v1 = eval_expression(exp1, stack, stack_start, false, "".to_string())?;
    let v2 = eval_expression(exp2, stack, stack_start, false, "".to_string())?;
    Result::Ok((v1, v2))
}

fn sum(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 + i2)),

        (Value::List(l1), Value::List(l2)) => {
            let mut list = l1.clone();
            list.extend(l2);
            Result::Ok(Value::List(list))
        }

        (Value::List(_), other) => return Result::Err(Error{msg: format!("cannot concatenate list to {}", other), v:Val(Unit)}),

        (other, Value::List(_)) => return Result::Err(Error{msg: format!("cannot concatenate {} to list", other), v:Val(Unit)}),

        (a, b) => Result::Ok(Value::Str(format!("{}{}", a, b))),
    }
}

fn sub(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 - i2)),
        _ => Result::Err(Error{msg: format!("Unsupported - operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn mul(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 * i2)),
        _ => Result::Err(Error{msg: format!("Unsupported * operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn div(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 / i2)),
        _ => Result::Err(Error{msg: format!("Unsupported / operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn modulo(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 % i2)),
        _ => Result::Err(Error{msg: format!("Unsupported % operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn lt(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 < i2)),
        _ => Result::Err(Error{msg: format!("Unsupported < operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn lte(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 <= i2)),
        _ => Result::Err(Error{msg: format!("Unsupported <= operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn gt(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 > i2)),
        _ => Result::Err(Error{msg: format!("Unsupported > operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn gte(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 >= i2)),
        _ => Result::Err(Error{msg: format!("Unsupported >= operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn eq(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 == i2)),
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(b1 == b2)),
        _ => Result::Err(Error{msg: format!("Unsupported == operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn neq(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 != i2)),
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(b1 != b2)),
        _ => Result::Err(Error{msg: format!("Unsupported != operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn and(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(*b1 && *b2)),
        _ => Result::Err(Error{msg: format!("Unsupported && operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}

fn or(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(*b1 || *b2)),
        _ => Result::Err(Error{msg: format!("Unsupported || operator for values {}, {}",val1, val2), v:Val(Unit)})
    }
}
