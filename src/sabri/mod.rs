use std::rc::Rc;

pub mod syntax;
pub mod bytecode;

pub mod value;
pub mod error;
pub mod env;
pub mod native;
pub mod symtab;

pub use self::symtab::SymTab;
pub use self::native::NativeFunc;
pub use self::value::Value;
pub use self::env::Env;
pub use self::error::{RunError, RunErrorValue};

pub type RunResult<T> = Result<T, RunError>;

pub struct Sabri {
    pub env:     Rc<Env>,
    pub sym_tab: Rc<SymTab>,
    pub bytecode: bytecode::Program,
}

impl Sabri {
    pub fn new() -> Sabri {
        let mut sabri = Sabri {
            env: Rc::new(Env::new_global()),
            sym_tab: Rc::new(SymTab::new_global()),
            bytecode: bytecode::Program::new(),
        };

        sabri.init_env();
        sabri
    }

    pub fn set_var(&mut self, var: &str, val: Value) {
        let index = self.sym_tab.add_name(var);
        if index >= self.env.size() {
            self.env.grow();
        }

        if let Err(e) = self.env.set_value(index, 0, val) {
            panic!("error setting variable: {}", e);
        }
    }

    pub fn get_var(&self, var: &str) -> Option<Value> {
        match self.sym_tab.get_name(var) {
            Some((i, env_index)) => {
                match self.env.get_value(i, env_index) {
                    Ok(v)  => Some(v),
                    Err(_) => None,
                }
            }
            
            None => None,
        }
    }

    pub fn init_env(&mut self) {
        self.set_var("null", Value::Null);
        self.set_var("true", Value::Bool(true));
        self.set_var("false", Value::Bool(false));

        self.set_var("putsf", Value::native_func(native::func_printf));
        self.set_var("putsl", Value::native_func(native::func_println));
        self.set_var("puts", Value::native_func(native::func_print));

        self.set_var("!",  Value::native_func(native::func_logic_not));
        self.set_var("==", Value::native_func(native::func_cmp_eq));
        self.set_var("!=", Value::native_func(native::func_cmp_ne));
        self.set_var("<",  Value::native_func(native::func_cmp_lt));
        self.set_var("<=", Value::native_func(native::func_cmp_le));
        self.set_var(">",  Value::native_func(native::func_cmp_gt));
        self.set_var(">=", Value::native_func(native::func_cmp_ge));
        self.set_var("+",  Value::native_func(native::func_num_add));
        self.set_var("-",  Value::native_func(native::func_num_sub));
        self.set_var("*",  Value::native_func(native::func_num_mul));
        self.set_var("/",  Value::native_func(native::func_num_div));
        self.set_var("^",  Value::native_func(native::func_num_pow));
        self.set_var("%",  Value::native_func(native::func_num_mod));
    }

    pub fn dump_bytecode(&self) {
        self.bytecode.dump()
    }
}