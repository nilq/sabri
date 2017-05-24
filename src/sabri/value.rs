use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

use sabri::{RunResult, RunError, NativeFunc, Env};
use sabri::bytecode;

#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(Rc<String>),
    NativeFunc(NativeFunc),
    Closure(bytecode::Closure),
}

impl Value {
    pub fn truthy(&self) -> bool {
        match *self {
            Value::Null      => false,
            Value::Bool(b)   => b,
            Value::Number(f) => f != 0f64,
            _ => true,
        }
    }

    pub fn native_func(f: fn(&[Value], &Rc<Env>) -> RunResult<Value>) -> Value {
        Value::NativeFunc(NativeFunc::new(f))
    }

    pub fn as_int(&self) -> RunResult<i64> {
        match *self {
            Value::Null          => Err(RunError::new("can't convert null to int")),
            Value::NativeFunc(_) => Err(RunError::new("can't convert native function to int")),
            Value::Closure(_)    => Err(RunError::new("can't convert closure to int")),
            Value::Bool(b)       => if b { Ok(1) } else { Ok(-1) },
            Value::Number(f)     => Ok(f as i64),
            Value::Str(ref s)    => match s.parse::<i64>() {
                Err(_) => Err(RunError::new(&format!("can't convert '{}' to int", s))),
                Ok(n)  => Ok(n),
            },
        }
    }

    pub fn as_float(&self) -> RunResult<f64> {
        match *self {
            Value::Null          => Err(RunError::new("can't convert null to float")),
            Value::NativeFunc(_) => Err(RunError::new("can't convert native function to float")),
            Value::Closure(_)    => Err(RunError::new("can't convert closure to float")),
            Value::Bool(b)       => if b { Ok(1f64) } else { Ok(-1f64) },
            Value::Number(f)     => Ok(f),
            Value::Str(ref s)    => match s.parse::<f64>() {
                Err(_) => Err(RunError::new(&format!("can't convert '{}' to float", s))),
                Ok(n)  => Ok(n),
            },
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null           => write!(f, "null"),
            Value::Bool(b)        => write!(f, "{}", b),
            Value::Number(n)      => write!(f, "{}", n),
            Value::Str(ref s)     => write!(f, "{}", s),
            Value::NativeFunc(n)  => write!(f, "{}", n),
            Value::Closure(ref c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null           => write!(f, "null"),
            Value::Bool(b)        => write!(f, "{}", b),
            Value::Number(n)      => write!(f, "{}", n),
            Value::Str(ref s)     => write!(f, "{}", s),
            Value::NativeFunc(n)  => write!(f, "{}", n),
            Value::Closure(ref c) => write!(f, "{}", c),
        }
    }
}