
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use sabri::{Value, RunResult, RunError};

pub struct Env {
    pub parent: Option<Rc<Env>>,
    values: RefCell<Vec<Value>>,
}

impl Env {
    pub fn new(parent: Rc<Env>, values: &[Value]) -> Env {
        Env {
            parent: Some(parent),
            values: RefCell::new(values.to_vec()),
        }
    }

    pub fn new_global() -> Env {
        Env {
            parent: None,
            values: RefCell::new(Vec::new()),
        }
    }

    pub fn new_partial(parent: Rc<Env>, values: &[Value], size: usize) -> Env {
        let mut stack = values.to_vec();
        for _ in 0 .. size - values.len() {
            stack.push(Value::Null)
        }

        Env {
            parent: Some(parent),
            values: RefCell::new(stack),
        }
    }

    pub fn set_value(&self, index: usize, env_index: usize, value: Value) -> RunResult<()> {
        if env_index == 0 {
            let mut values = self.values.borrow_mut();
            match values.get_mut(index) {
                Some(v) => {
                    *v = value;
                    Ok(())
                },
                None => Err(RunError::new(&format!("can't set value of invalid value index: {}", index))),
            }
        } else {
            match self.parent {
                Some(ref p) => p.set_value(index, env_index - 1, value),
                None => Err(RunError::new(&format!("can't set value with invalid env index: {}", env_index))),
            }
        }
    }

    pub fn get_value(&self, index: usize, env_index: usize) -> RunResult<Value> {
        if env_index == 0 {
            match self.values.borrow().get(index) {
                Some(v) => Ok(v.clone()),
                None    => Err(RunError::new(&format!("can't get value of invalid value index: {}", index))),
            }
        } else {
            match self.parent {
                Some(ref p) => p.get_value(index, env_index - 1),
                None => Err(RunError::new(&format!("can't get value with invalid env index: {}", index))),
            }
        }
    }

    fn dump(&self, f: &mut fmt::Formatter, env_index: usize) -> fmt::Result {
        if let Some(ref p) = self.parent {
            try!(p.dump(f, env_index - 1));
            try!(writeln!(f, "------------------------------"));
        }

        for (i, v) in self.values.borrow().iter().enumerate() {
            try!(writeln!(f, "<{}@{}> {}", i, env_index, v))
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.values.borrow().len()
    }

    pub fn grow(&self) {
        self.values.borrow_mut().push(Value::Null)
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}