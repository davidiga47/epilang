use crate::expression::Exp;
use crate::parser::SyntaxError;
use crate::value::{Value, StackValue, Function, V};
use substring::Substring;

pub struct Error {
    pub msg: String
}

pub fn eval(exp: &Exp) -> Result<V, Error> {
    let mut stack: Vec<StackValue> = Vec::new();
    eval_expression(exp, &mut stack, 0)
}

pub fn eval_expression(exp: &Exp, stack: &mut Vec<StackValue>, stack_start: usize) -> Result<V, Error> {
    match exp {
        Exp::Const(c) => Result::Ok(V::Val(Value::from_const(&c))),

        Exp::Var(x) => {
            Result::Ok(V::Ptr(stack[x.scope + stack_start]))
        },

        Exp::Decl(_, val_exp, exp2) => {
            match eval_expression(val_exp, stack, stack_start)? {
                V::Ptr(ptr) => stack.push(ptr),
                V::Val(value) => stack.push(StackValue::from_box(Box::new(value)))
            };
            let result = eval_expression(exp2, stack, stack_start);
            stack.pop();
            result
        },

        Exp::List(list) => {
            let mut values: Vec<StackValue> = Vec::with_capacity(list.len());
            for exp in list {
                let value = match eval_expression(exp, stack, stack_start)? {
                    V::Ptr(ptr) => ptr,
                    V::Val(value) => StackValue::from_box(Box::new(value))
                };
                values.push(value)
            }
            Result::Ok(V::Val(Value::List(values)))
        }

        Exp::ListSelection(list, index) => {
            let list: V = eval_expression(list, stack, stack_start)?;
            let index: V = eval_expression(index, stack, stack_start)?;
            let value = match (list.as_ref(), index.as_ref()) {
                (Value::List(values), Value::Int(i)) => values.get(*i as usize)
                    .ok_or(Error{msg: String::from("List index out of range")})?,
                _ => return Result::Err(Error{msg: String::from("q")})
            };
            Result::Ok(V::Ptr(*value))
        }

        Exp::Assign(left_exp, right_exp) => {
            let right_value: V = eval_expression(right_exp, stack, stack_start)?;
            match (*left_exp).as_ref() {
                Exp::Var(var) => match right_value {
                    V::Ptr(ptr) => stack[var.scope + stack_start] = ptr,
                    V::Val(value) => stack[var.scope + stack_start] = StackValue::from_box(Box::new(value))
                },
                Exp::ListSelection(list, index) => {
                    let mut list = eval_expression(list.as_ref(), stack, stack_start)?;
                    let list: &mut Vec<StackValue> = match list.as_mut_ref() {
                        Value::List(list) => list,
                        _ => return Result::Err(Error{msg: String::from("Expected list value before list selection")})
                    };
                    let index: usize = match eval_expression(index.as_ref(), stack, stack_start)?.as_ref() {
                        Value::Int(i) => *i as usize,
                        _ => return Result::Err(Error{msg: String::from("Expected number in list selection")})
                    };
                    if index >= list.len() {
                        return Result::Err(Error{msg: String::from("List index out of range")})
                    }
                    list[index] = match right_value {
                        V::Ptr(ptr) => ptr,
                        V::Val(value) => StackValue::from_box(Box::new(value))
                    }
                },
                _ => return Result::Err(Error{msg: String::from("Invalid left-hand side in assignment")})
            }
            Result::Ok(V::Ptr(StackValue::unit()))
        }

        Exp::While(guard, exp) => {
            let condition = eval_expression(guard, stack, stack_start)?.as_bool();
            if !condition {
                Result::Ok(V::Val(Value::Unit))
            } else {
                loop {
                    let v: V = eval_expression(exp, stack, stack_start)?;
                    if !eval_expression(guard, stack, stack_start)?.as_bool() {
                        break Result::Ok(v)
                    }
                }
            }
        }

        Exp::IfThenElse(condition, exp1, exp2) => {
            let is_true: bool = eval_expression(condition, stack, stack_start)?.as_bool();
            eval_expression(if is_true {exp1} else {exp2}, stack, stack_start)
        }

        Exp::TryCatch(exp1,exc,exp2) => {
            let res: Result<V, Error> = eval_expression(exp1,stack,stack_start);
            match res {
                Result::Ok(_) => return Result::Ok(res?),
                Result::Err(Error{msg}) => {
                    let s1: String = "uncaught exception ".to_string();
                    let e=eval_expression(exc,stack,stack_start)?.as_string();
                    let s2: String =s1+&e;
                    if s2.eq(&msg){
                        let v: V = eval_expression(exp2, stack, stack_start)?;
                        return Result::Ok(v)
                    } else {
                        return Result::Err(Error{msg})
                    }
                }
            }
        }

        //evaluate the exception then returns the string containing the value. Try-Catch, if present, will try to catch the string
        Exp::Throw(exp) => {
            let res=eval_expression(exp,stack,stack_start)?;
            return Result::Err(Error{msg: String::from("uncaught exception ".to_string()+&res.to_string())})
        },

        Exp::Throwcc(k,e) => {
            let res=eval_expression(e,stack,stack_start)?;
            return Result::Err(Error{msg: String::from(&res.to_string())})
        },

        Exp::Callcc(k,e) => {
            /*
            let result: Result<V, Error> = eval_expression(e,stack,stack_start);
            match result {
                Result::Ok(v) => return Result::Ok(v),
                Result::Err(err) => return Result::Err(err)
            }*/
            let res: Result<V, Error> = eval_expression(e,stack,stack_start);
            match res {
                Result::Ok(_) => return Result::Ok(res?),
                Result::Err(Error{msg}) => {
                    if (msg.len()>18 && msg.substring(0,18).eq(&"uncaught exception".to_string())) { return Result::Err(Error{msg: msg})};
                    let numb: isize = msg.parse().unwrap();
                    return Result::Ok(V::Val(Value::Int(numb)))
                }
                
            }
        },

        Exp::Function(args, body) => {
            Result::Ok(V::Val(Value::Fn(Function { num_args: args.len(), external_values: Vec::new(), body: body.clone() })))
        },

        Exp::FunctionCall(callable, args) => {
            match eval_expression(callable, stack, stack_start)?.as_ref() {
                Value::Fn(function) => {
                    if args.len() != function.num_args {
                        return Result::Err(Error { msg: format!("Wrong number of arguments. Expected {}, found {}", function.num_args, args.len()) })
                    }
                    let function_stack_start: usize = stack.len();
                    for arg in args {
                        match eval_expression(arg, stack, stack_start)? {
                            V::Ptr(ptr) => stack.push(ptr),
                            V::Val(value) => stack.push(StackValue::from_box(Box::new(value)))
                        };
                    };
                    let result: V = eval_expression(function.body.as_ref(), stack, function_stack_start)?;
                    for _ in 0..function.num_args {
                        stack.pop();
                    }
                    Result::Ok(result)
                },
                _ => return Result::Err(Error{msg: String::from("Expression is not callable")})
            }
        },

        Exp::Seq(exp1, exp2) => {
            eval_expression(exp1, stack, stack_start)?;
            eval_expression(exp2, stack, stack_start)
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
            let val_exp1 = eval_expression(exp1, stack, stack_start)?;
            if (!val_exp1.as_bool()) {
                Result::Ok(V::Val(Value::Bool(false)))
            }
            else {
                let val_exp2 = eval_expression(exp2, stack, stack_start)?;
                Result::Ok(V::Val(Value::Bool(val_exp2.as_bool())))
            }
        },

        Exp::Or(exp1, exp2) => {
            let val_exp1 = eval_expression(exp1, stack, stack_start)?;
            if (val_exp1.as_bool()) {
                Result::Ok(V::Val(Value::Bool(true)))
            }
            else {
                let val_exp2 = eval_expression(exp2, stack, stack_start)?;
                Result::Ok(V::Val(Value::Bool(val_exp2.as_bool())))
            }
        },

        Exp::Not(exp1) => {
            let v = eval_expression(exp1, stack, stack_start)?;
            Result::Ok(V::Val(Value::Bool(!v.as_bool())))
        }
    }
}

fn double_eval(exp1: &Exp, exp2: &Exp, stack: &mut Vec<StackValue>, stack_start: usize) -> Result<(V, V), Error> {
    let v1 = eval_expression(exp1, stack, stack_start)?;
    let v2 = eval_expression(exp2, stack, stack_start)?;
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

        (Value::List(_), other) => return Result::Err(Error{msg: format!("cannot concatenate list to {}", other)}),

        (other, Value::List(_)) => return Result::Err(Error{msg: format!("cannot concatenate {} to list", other)}),

        (a, b) => Result::Ok(Value::Str(format!("{}{}", a, b))),
    }
}

fn sub(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 - i2)),
        _ => Result::Err(Error{msg: format!("Unsupported - operator for values {}, {}",val1, val2)})
    }
}

fn mul(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 * i2)),
        _ => Result::Err(Error{msg: format!("Unsupported * operator for values {}, {}",val1, val2)})
    }
}

fn div(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 / i2)),
        _ => Result::Err(Error{msg: format!("Unsupported / operator for values {}, {}",val1, val2)})
    }
}

fn modulo(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Int(i1 % i2)),
        _ => Result::Err(Error{msg: format!("Unsupported % operator for values {}, {}",val1, val2)})
    }
}

fn lt(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 < i2)),
        _ => Result::Err(Error{msg: format!("Unsupported < operator for values {}, {}",val1, val2)})
    }
}

fn lte(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 <= i2)),
        _ => Result::Err(Error{msg: format!("Unsupported <= operator for values {}, {}",val1, val2)})
    }
}

fn gt(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 > i2)),
        _ => Result::Err(Error{msg: format!("Unsupported > operator for values {}, {}",val1, val2)})
    }
}

fn gte(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 >= i2)),
        _ => Result::Err(Error{msg: format!("Unsupported >= operator for values {}, {}",val1, val2)})
    }
}

fn eq(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 == i2)),
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(b1 == b2)),
        _ => Result::Err(Error{msg: format!("Unsupported == operator for values {}, {}",val1, val2)})
    }
}

fn neq(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Int(i1), Value::Int(i2)) => Result::Ok(Value::Bool(i1 != i2)),
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(b1 != b2)),
        _ => Result::Err(Error{msg: format!("Unsupported != operator for values {}, {}",val1, val2)})
    }
}

fn and(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(*b1 && *b2)),
        _ => Result::Err(Error{msg: format!("Unsupported && operator for values {}, {}",val1, val2)})
    }
}

fn or(val1: &Value, val2: &Value) -> Result<Value, Error> {
    match (val1, val2) {
        (Value::Bool(b1), Value::Bool(b2)) => Result::Ok(Value::Bool(*b1 || *b2)),
        _ => Result::Err(Error{msg: format!("Unsupported || operator for values {}, {}",val1, val2)})
    }
}
