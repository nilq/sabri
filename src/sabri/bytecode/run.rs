use std::rc::Rc;

use sabri::bytecode;
use sabri::native;

use self::bytecode::{Env, Value};
use self::bytecode::{RunError, RunResult};
use self::bytecode::op::*;
use self::bytecode::instr;

use self::bytecode::{Addr, INVALID};

pub struct Run {
    ip: u32,
    env: Rc<Env>,
    env_stack: Vec<Rc<Env>>,
    val_stack: Vec<Value>,
    ret_stack: Vec<u32>,
    flag: bool,
}

impl Run {
    pub fn new(env: Rc<Env>) -> Run {
        Run {
            env,
            ip: 0,
            env_stack: vec![],
            val_stack: vec![],
            ret_stack: vec![],
            flag: false,
        }
    }

    pub fn reset(&mut self, env: Rc<Env>) {
        self.env = env;
        self.ip  = 0;
        self.env_stack.clear();
        self.val_stack.clear();
        self.ret_stack.clear();
        self.flag = false;
    }

    pub fn exec(&mut self, n: usize, instr: &[u32], literals: &[Value]) -> RunResult<()> {
        for _ in 0 .. n {
            if self.ip == INVALID || self.ip >= instr.len() as u32 {
                break
            }

            let instr = instr[self.ip as usize];
            let op    = (instr >> 26) as u8;

            match op {
                HALT => self.ip = INVALID,
                PUSHLIT => {
                    let index = instr::d_op_26(instr);
                    self.val_stack.push(literals[index as usize].clone());
                    self.ip += 1;
                },
                NEWENV => {
                    let (args_n, total) = instr::d_op_12_12(instr);
                    let args_n = args_n as usize;

                    let start = self.val_stack.len() - args_n;

                    {
                        let args = &self.val_stack[start..];
                        
                        self.env_stack.push(self.env.clone());
                        self.env = Rc::new(Env::new_partial(self.env.clone(), args, total as usize));
                    }
                    
                    self.val_stack.drain(start..);
                    self.ip += 1
                },
                POPENV => {
                    let envs = instr::d_op_12(instr) as usize;
                    for _ in 0 .. envs {
                        self.env = match self.env_stack.pop() {
                            Some(e) => e,
                            None    => return Err(RunError::new("popping env on empty stack")),
                        }
                    }
                    self.ip += 1
                },
                POPVAL => {
                    let vals = instr::d_op_12(instr) as usize;
                    for _ in 0 .. vals {
                        match self.val_stack.pop() {
                            Some(_) => (),
                            None    => return Err(RunError::new("popping value on empty stack")),
                        }
                    }
                    self.ip += 1
                },
                GETVAR => {
                    let (i, env_index) = instr::d_op_12_12(instr);
                    match self.env.get_value(i as usize, env_index as usize) {
                        Ok(v) => self.val_stack.push(v.clone()),
                        Err(e) => return Err(e),
                    }
                    self.ip += 1
                },
                SETVAR => {
                    let (i, env_index) = instr::d_op_12_12(instr);
                    let val = match self.val_stack.pop() {
                        Some(v) => v,
                        None => return Err(RunError::new("setting var on empty value stack")),
                    };
                    match self.env.set_value(i as usize, env_index as usize, val.clone()) {
                        Ok(_)   => self.val_stack.push(val),
                        Err(e) => {
                            return Err(e);
                        }
                    }
                    self.ip += 1
                },
            
                RET => {
                    self.env = match self.env_stack.pop() {
                        Some(e) => e,
                        None    => return Err(RunError::new("returning on empty env stack"))
                    };

                    self.ip = match self.ret_stack.pop() {
                        Some(a) => a,
                        None    => return Err(RunError::new("returning on empty ret stack"))
                    };
                }

                TEST => {
                    let value = match self.val_stack.pop() {
                        Some(e) => e,
                        None    => return Err(RunError::new("testing on empty value stack")),
                    };

                    self.flag = value.truthy();
                    self.ip += 1
                }

                JMP => self.ip = instr::d_op_26(instr),

                JT => if self.flag {
                    self.ip = instr::d_op_26(instr)
                } else {
                    self.ip += 1
                },

                JF => if !self.flag {
                    self.ip = instr::d_op_26(instr)
                } else {
                    self.ip += 1
                },

                ADD | SUB | MUL | DIV => {
                    if self.val_stack.len() < 2 {
                        return Err(RunError::new("can't operate with less than two values"));
                    }

                    let args_pos = self.val_stack.len() - 2;

                    let result = match op {
                        ADD => native::func_num_add(&self.val_stack[args_pos..], &self.env),
                        SUB => native::func_num_sub(&self.val_stack[args_pos..], &self.env),
                        MUL => native::func_num_mul(&self.val_stack[args_pos..], &self.env),
                        DIV => native::func_num_div(&self.val_stack[args_pos..], &self.env),

                        _ => return Err(RunError::new("internal error: unhandled arithmetic op")),
                    };

                    println!("result: {:#?}", result);

                    self.val_stack.drain(args_pos..);
                    self.val_stack.push(try!(result));

                    self.ip += 1
                },
            
                _ => {
                    println!("warning: unhandled bytecode at: {:08x}", self.ip);
                    self.ip = INVALID;
                    break
                },
            }
        }
        Ok(())
    }
}