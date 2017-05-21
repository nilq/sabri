use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;

use sabri::{RunResult, RunError};

#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<String>),
}

impl Value {
    pub fn truthy(&self) -> bool {
        match *self {
            Value::Null     => false,
            Value::Bool(b)  => b,
            Value::Int(i)   => i != 0i64,
            Value::Float(f) => f != 0f64,
            _ => true,
        }
    }

    pub fn as_int(&self) -> RunResult<i64> {
        match *self {
            Value::Null       => Err(RunError::new("can't convert null to int")),
            Value::Bool(b)    => if b { Ok(1) } else { Ok(-1) },
            Value::Int(i)     => Ok(i),
            Value::Float(f)   => Ok(f as i64),
            Value::Str(ref s) => match s.parse::<i64>() {
                Err(_) => Err(RunError::new(&format!("can't convert '{}' to int", s))),
                Ok(n)  => Ok(n),
            },
        }
    }

    pub fn as_flot(&self) -> RunResult<f64> {
        match *self {
            Value::Null       => Err(RunError::new("can't convert null to float")),
            Value::Bool(b)    => if b { Ok(1f64) } else { Ok(-1f64) },
            Value::Int(i)     => Ok(i as f64),
            Value::Float(f)   => Ok(f),
            Value::Str(ref s) => match s.parse::<f64>() {
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
            Value::Null       => write!(f, "null"),
            Value::Bool(b)    => write!(f, "{}", b),
            Value::Int(i)     => write!(f, "{}", i),
            Value::Float(fl)  => write!(f, "{}", fl),
            Value::Str(ref s) => write!(f, "{}", s),
        }
    }
}