
use std::fmt;
use std::cmp;
use std::rc::Rc;

use sabri::{Value, RunError, RunResult};
use sabri::Env;

#[derive(Copy)]
pub struct NativeFunc {
    pub f: fn(&[Value], &Rc<Env>) -> RunResult<Value>,
}

impl NativeFunc {
    pub fn new(f: fn(&[Value], &Rc<Env>) -> RunResult<Value>) -> NativeFunc {
        NativeFunc { f: f }
    }

    pub fn call(&self, args: &[Value], env: &Rc<Env>) -> RunResult<Value> {
        match (self.f)(args, env) {
            Err(e) => Err(RunError::new("failed to call native")),
            Ok(x) => Ok(x),
        }
    }
}

impl Clone for NativeFunc {
    fn clone(&self) -> NativeFunc {
        NativeFunc { f: self.f }
    }
}

impl fmt::Display for NativeFunc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native_func@{:x}>", self as *const NativeFunc as usize)
    }
}

impl cmp::PartialEq for NativeFunc {
    fn eq(&self, other: &NativeFunc) -> bool {
        self as *const NativeFunc == other as *const NativeFunc
    }
}

fn get_arg(args: &[Value], index: usize) -> RunResult<&Value> {
    match args.get(index) {
        Some(v) => Ok(v),
        None => {
            Err(RunError::new(&format!("expected argument at position {}", index + 1)))
        }
    }
}

fn cmp_eq(args: &[Value]) -> RunResult<bool> {
    let left = try!(get_arg(args, 0));
    let right = try!(get_arg(args, 1));

    let b = match (left, right) {
        (&Value::Null, &Value::Null) => true,
        (&Value::Bool(l), &Value::Bool(r)) => l == r,
        (&Value::Number(l), &Value::Number(r)) => l == r,
        (&Value::Str(ref l), &Value::Str(ref r)) => l == r,
        (&Value::NativeFunc(ref l), &Value::NativeFunc(ref r)) => l == r,
        _ => false,
    };
    Ok(b)
}

fn cmp_order(args: &[Value],
             num_cmp: fn(f64, f64) -> bool,
             str_cmp: fn(&str, &str) -> bool)
             -> RunResult<bool> {
    let left = try!(get_arg(args, 0));
    let right = try!(get_arg(args, 1));

    let b = match (left, right) {
        (&Value::Number(l), &Value::Number(r)) => num_cmp(l, r),
        (&Value::Str(ref l), &Value::Str(ref r)) => str_cmp(l, r),
        _ => false,
    };
    Ok(b)
}

fn cmp_lt_num(l: f64, r: f64) -> bool {
    l < r
}

fn cmp_le_num(l: f64, r: f64) -> bool {
    l <= r
}

fn cmp_lt_str(l: &str, r: &str) -> bool {
    l < r
}
fn cmp_le_str(l: &str, r: &str) -> bool {
    l <= r
}

fn bin_arithmetic(args: &[Value], op: fn(f64, f64) -> f64, name: &str) -> RunResult<f64> {
    let left = try!(get_arg(args, 0));
    let right = try!(get_arg(args, 1));
    match (left, right) {
        (&Value::Number(l), &Value::Number(r)) => Ok(op(l, r)),
        _ => Err(RunError::new(&format!("invalid arguments for '{}'", name))),
    }
}

fn num_add(l: f64, r: f64) -> f64 {
    l + r
}

fn num_mul(l: f64, r: f64) -> f64 {
    l * r
}

fn num_sub(l: f64, r: f64) -> f64 {
    l - r
}

fn num_div(l: f64, r: f64) -> f64 {
    l / r
}

fn num_pow(l: f64, r: f64) -> f64 {
    l.powf(r)
}

fn num_mod(l: f64, r: f64) -> f64 {
    l - (l / r).trunc() * r
}

fn un_arithmetic(args: &[Value], op: fn(f64) -> f64, name: &str) -> RunResult<f64> {
    let arg = try!(get_arg(args, 0));
    match *arg {
        Value::Number(x) => Ok(op(x)),
        _ => Err(RunError::new(&format!("invalid argument for '{}'", name))),
    }
}

fn num_neg(x: f64) -> f64 {
    -x
}

pub fn func_dump_env(_args: &[Value], env: &Rc<Env>) -> RunResult<Value> {
    println!("{:?}", env);
    Ok(Value::Null)
}

pub fn func_error(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    if let Some(v) = args.get(0) {
        Err(RunError::new("native not provided enough args"))
    } else {
        Ok(Value::Null)
    }
}

pub fn func_printf(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    if let Some(&Value::Str(ref fmt)) = args.get(0) {
        let mut chars = fmt.chars();
        let mut next_arg = 1;
        while let Some(ch) = chars.next() {
            if ch == '%' {
                match chars.next() {
                    Some('%') => print!("%"),

                    Some('x') => {
                        print!("{:x}", try!(try!(get_arg(args, next_arg)).as_int()));
                        next_arg += 1;
                    }
                    Some('d') => {
                        print!("{}", try!(try!(get_arg(args, next_arg)).as_int()));
                        next_arg += 1;
                    }
                    Some('f') => {
                        print!("{}", try!(try!(get_arg(args, next_arg)).as_float()));
                        next_arg += 1;
                    }
                    Some('s') => {
                        print!("{}", try!(get_arg(args, next_arg)));
                        next_arg += 1;
                    }

                    Some(c) => {
                        return Err(RunError::new(&format!("invalid format specifier: {:?}",
                                                                    c)))
                    }
                    None => return Err(RunError::new("expected format specifier")),
                };
            } else {
                print!("{}", ch);
            }
        }
        Ok(Value::Null)
    } else {
        Err(RunError::new("expected format string"))
    }
}

pub fn func_logic_not(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    let val = try!(get_arg(args, 0));
    Ok(Value::Bool(!val.truthy()))
}

pub fn func_cmp_eq(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Bool(try!(cmp_eq(args))))
}

pub fn func_cmp_ne(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Bool(!try!(cmp_eq(args))))
}

pub fn func_cmp_lt(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Bool(try!(cmp_order(args, cmp_lt_num, cmp_lt_str))))
}

pub fn func_cmp_le(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Bool(try!(cmp_order(args, cmp_le_num, cmp_le_str))))
}

pub fn func_cmp_gt(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Bool(!try!(cmp_order(args, cmp_le_num, cmp_le_str))))
}

pub fn func_cmp_ge(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Bool(!try!(cmp_order(args, cmp_lt_num, cmp_lt_str))))
}

pub fn func_num_add(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_add, "+"))))
}

pub fn func_num_sub(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    if args.len() == 1 {
        return Ok(Value::Number(try!(un_arithmetic(args, num_neg, "-"))));
    }
    Ok(Value::Number(try!(bin_arithmetic(args, num_sub, "-"))))
}

pub fn func_num_mul(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_mul, "*"))))
}

pub fn func_num_div(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_div, "/"))))
}

pub fn func_num_pow(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_pow, "^"))))
}

pub fn func_num_mod(args: &[Value], _env: &Rc<Env>) -> RunResult<Value> {
    Ok(Value::Number(try!(bin_arithmetic(args, num_mod, "%"))))
}
