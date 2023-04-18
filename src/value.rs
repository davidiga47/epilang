use std::fmt;
use std::ptr;

use crate::expression::Exp;
use crate::expression::Const;

#[derive(Debug)]
pub struct Function {
    pub num_args: usize,
    pub external_values: Vec<Value>,
    pub body: Box<Exp>
}

#[derive(Copy, Clone, Debug)]
pub struct StackValue {
    pub value: *mut Value,
}

impl StackValue {
    pub fn from_box(val: Box<Value>) -> StackValue {
        StackValue { value: Box::into_raw(val) }
    }

    pub fn unit() -> StackValue {
        StackValue{value: ptr::null_mut()}
    }

    pub fn is_unit(&self) -> bool {
        self.value.is_null()
    }

    pub fn as_ref(&self) -> &Value {
        unsafe{ &*self.value }
    }

    pub fn as_mut_ref(&mut self) -> &mut Value {
        unsafe{ &mut *self.value }
    }
}

impl fmt::Display for StackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug)]
pub enum Value {
    Unit,
    Int(isize),
    Bool(bool),
    Fn(Function),
    List(Vec<StackValue>),
    Str(String)
}

impl Value {
    pub fn from_const(c: &Const) -> Value {
        match c {
            Const::None => Value::Unit,
            Const::Integer(i) => Value::Int(isize::try_from(*i).ok().unwrap()),
            Const::Boolean(b) => Value::Bool(*b),
            Const::String(s) => Value::Str(s.clone())
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Unit => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Str(s) => s == "",
            Value::Fn(_) => true,
            Value::List(_) => true
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Value::Unit => String::from("Unit"),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Str(s) => s.to_string(),
            Value::Fn(_) => String::from("Function"),
            Value::List(_) => String::from("List")
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Unit, Value::Unit) => true,
            (Value::Int(i1), Value::Int(i2)) => *i1 == *i2,
            (Value::Bool(b1), Value::Bool(b2)) => *b1 == *b2,
            (Value::Str(s1), Value::Str(s2)) => s1 == s2,
            (Value::Fn(f1), Value::Fn(f2)) => std::ptr::eq(f1, f2),
            _ => false
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "unit"),
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Fn(func) => write!(f, "{:?}", func),
            Value::List(list) => {
                write!(f, "[")?;
                if list.len() > 0 {
                    for i in 0..list.len() -1 {
                        write!(f, "{}, ", list[i])?;
                    };
                    write!(f, "{}", list[list.len() - 1])?;
                }
                write!(f, "]")
            }
        }
    }
}

///////////

#[derive(Debug)]
pub enum V {
    Val(Value),
    Ptr(StackValue)
}

impl V {
    pub fn as_bool(&self) -> bool {
        match self {
            V::Ptr(ptr) => if ptr.is_unit() {false} else {ptr.as_ref().as_bool()},
            V::Val(value) => value.as_bool()
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            V::Ptr(ptr) => ptr.as_ref().as_string(),
            V::Val(value) => value.as_string()
        }
    }

    pub fn as_ref(&self) -> &Value {
        match self {
            V::Ptr(ptr) => ptr.as_ref(),
            V::Val(value) => value
        }
    }

    pub fn as_mut_ref(&mut self) -> &mut Value {
        match self {
            V::Ptr(ptr) => ptr.as_mut_ref(),
            V::Val(value) => value
        }
    }
}

impl PartialEq for V {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<Value> for V {
    fn eq(&self, other: &Value) -> bool {
        self.as_ref() == other
    }
}

impl fmt::Display for V {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            V::Ptr(ptr) => if ptr.is_unit() {write!(f, "unit")}  else {write!(f, "{}", ptr.as_ref())},
            V::Val(value) => write!(f, "{}", value)
        }
    }
}
