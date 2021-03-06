use std::collections::HashMap;

use sabri::bytecode;

use self::bytecode::{ParserResult, ParserError};
use self::bytecode::TokenPosition;
use self::bytecode::Value;

use self::bytecode::op::*;
use self::bytecode::instr;

use self::bytecode::Addr;

pub struct FixupContext {
    pub init_level: u32,
    pub instr_addrs: Vec<Addr>,
}

impl FixupContext {
    pub fn new(init_level: u32) -> FixupContext {
        FixupContext {
            init_level,
            instr_addrs: vec![],
        }
    }

    pub fn add(&mut self, instr_addr: Addr) -> ParserResult<()> {
        self.instr_addrs.push(instr_addr);
        Ok(())
    }

    pub fn close(self, instr: &mut [u32], fixed_addr: Addr) -> ParserResult<()> {
        for addr in self.instr_addrs {
            instr[addr as usize] = instr::f_op_26(instr[addr as usize], fixed_addr)
        }
        Ok(())
    }
}

pub struct Program {
    pub instr: Vec<u32>,
    pub literals: Vec<Value>,

    while_context: Vec<FixupContext>,
    func_context: Vec<FixupContext>,

    env_level: u32,

    labels: HashMap<Addr, String>,
    comments: HashMap<Addr, String>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            instr: vec![],
            literals: vec![Value::Null],
            while_context: vec![],
            func_context: vec![],
            env_level: 0,
            labels: HashMap::new(),
            comments: HashMap::new(),
        }
    }

    pub fn addr(&self) -> Addr {
        self.instr.len() as Addr
    }

    pub fn add_label(&mut self, addr: Addr, comment: &str) {
        self.labels.insert(addr, comment.to_string());
    }

    pub fn increment_env_level(&mut self, n: u32) {
        self.env_level += n;
    }

    pub fn decrement_env_level(&mut self, n: u32) -> ParserResult<()> {
        if self.env_level < n {
            Err(ParserError::new("can't add fixup context without creating context"))
        } else {
            self.env_level -= n;
            Ok(())
        }
    }

    pub fn get_env_level(&mut self) -> u32 {
        self.env_level
    }

    pub fn set_env_level(&mut self, n: u32) {
        self.env_level = n;
    }

    pub fn add_comment(&mut self, comment: &str) {
        let addr = self.addr();
        self.comments.insert(addr, comment.to_string());
    }

    pub fn add_literal(&mut self, val: Value) -> usize {
        let index = self.literals.len();
        self.literals.push(val);
        index
    }

    pub fn new_func_context(&mut self) {
        let env_level = self.env_level;
        self.func_context.push(FixupContext::new(env_level));
    }

    pub fn close_func_context(&mut self, fixed_addr: Addr) -> ParserResult<()> {
        match self.func_context.pop() {
            Some(c) => c.close(&mut self.instr, fixed_addr),
            None => Err(ParserError::new("missing function context to close")),
        }
    }

    pub fn add_return_fixup(&mut self, addr: Addr) -> ParserResult<()> {
        match self.func_context.last_mut() {
            None => Err(ParserError::new("can't add return fixup to nothing")),
            Some(c) => c.add(addr),
        }
    }

    pub fn new_while_context(&mut self) {
        let env_level = self.env_level;
        self.while_context.push(FixupContext::new(env_level));
    }

    pub fn close_while_context(&mut self, fixed_addr: Addr) -> ParserResult<()> {
        match self.while_context.pop() {
            Some(c) => c.close(&mut self.instr, fixed_addr),
            None => Err(ParserError::new("can't close non-existing while context")),
        }
    }

    pub fn add_break_fixup(&mut self, addr: Addr) -> ParserResult<()> {
        match self.while_context.last_mut() {
            Some(c) => c.add(addr),
            None => Err(ParserError::new("can't add break fixup to nothing")),
        }
    }

    pub fn get_while_env_level(&self) -> ParserResult<u32> {
        let env_level = self.env_level;
        match self.while_context.last() {
            Some(c) => Ok(env_level - c.init_level),
            None => Err(ParserError::new("can't operate while-context outside while-context")),
        }
    }

    pub fn fix_newenv(&mut self, instr_addr: Addr, n_vals: u16, n_total: u16) {
        self.instr[instr_addr as usize] =
            instr::f_op_12_12(self.instr[instr_addr as usize], n_vals, n_total);
    }

    pub fn fix_jump(&mut self, instr_addr: Addr, target_addr: Addr) {
        self.instr[instr_addr as usize] = instr::f_op_26(self.instr[instr_addr as usize],
                                                         target_addr);
    }

    pub fn emit_halt(&mut self) {
        self.instr.push(instr::c_op_26(HALT, 0x3ffffff));
    }

    pub fn emit_newenv(&mut self, params: u16, total: u16) {
        self.instr.push(instr::c_op_12_12(NEWENV, params, total));
    }

    pub fn emit_popenv(&mut self, envs: u16) {
        self.instr.push(instr::c_op_12(POPENV, envs));
    }

    pub fn emit_getvar(&mut self, index: u16, env_index: u16) {
        self.instr.push(instr::c_op_12_12(GETVAR, index, env_index));
    }

    pub fn emit_setvar(&mut self, index: u16, env_index: u16) {
        self.instr.push(instr::c_op_12_12(SETVAR, index, env_index));
    }

    pub fn emit_getelem(&mut self) {
        self.instr.push(instr::c_op(GETELEM));
    }

    pub fn _emit_setelem(&mut self) {
        self.instr.push(instr::c_op(SETELEM));
    }

    pub fn emit_pushlit(&mut self, index: usize) {
        self.instr.push(instr::c_op_26(PUSHLIT, index as u32));
    }

    pub fn emit_add(&mut self) {
        self.instr.push(instr::c_op(ADD));
    }

    pub fn emit_sub(&mut self) {
        self.instr.push(instr::c_op(SUB));
    }

    pub fn emit_mul(&mut self) {
        self.instr.push(instr::c_op(MUL));
    }

    pub fn emit_div(&mut self) {
        self.instr.push(instr::c_op(DIV));
    }

    pub fn emit_test(&mut self) {
        self.instr.push(instr::c_op(TEST));
    }

    pub fn emit_jmp(&mut self, addr: Addr) {
        self.instr.push(instr::c_op_26(JMP, addr));
    }

    pub fn _emit_jt(&mut self, addr: Addr) {
        self.instr.push(instr::c_op_26(JT, addr));
    }

    pub fn emit_jf(&mut self, addr: Addr) {
        self.instr.push(instr::c_op_26(JF, addr));
    }

    pub fn emit_call(&mut self, args: u16) {
        self.instr.push(instr::c_op_12(CALL, args));
    }

    pub fn emit_ret(&mut self) {
        self.instr.push(instr::c_op(RET));
    }

    pub fn emit_popval(&mut self, values: u16) {
        self.instr.push(instr::c_op_12(POPVAL, values));
    }

    pub fn dump(&self) {
        println!("================================================");
        println!("==== INSTRUCTIONS");
        for (addr, &instr) in self.instr.iter().enumerate() {
            if let Some(label) = self.labels.get(&(addr as u32)) {
                println!("");
                println!(".{}:", label);
            }
            print!("{:08x}:   {:08x}   ", addr, instr);

            match (instr >> 26) as u8 {
                OP_HALT => print!("halt       "),

                OP_NEWENV => {
                    print!("newenv     {}, {}",
                           instr::d_op_12_12(instr).0,
                           instr::d_op_12_12(instr).1)
                }
                
                OP_POPENV => print!("popenv     {}", instr::d_op_12(instr)),

                OP_GETVAR => {
                    print!("getvar     {}, {}",
                           instr::d_op_12_12(instr).0,
                           instr::d_op_12_12(instr).1)
                }

                OP_SETVAR => {
                    print!("setvar     {}, {}",
                           instr::d_op_12_12(instr).0,
                           instr::d_op_12_12(instr).1)
                }

                OP_GETELEM => print!("getelem    "),
                OP_SETELEM => print!("setelem    "),
                OP_PUSHLIT => print!("pushlit    {}", instr::d_op_26(instr)),

                OP_ADD => print!("add        "),
                OP_SUB => print!("sub        "),
                OP_MUL => print!("mul        "),
                OP_DIV => print!("div        "),

                OP_TEST => print!("test       "),
                OP_JMP => print!("jmp        {:08x}", instr::d_op_26(instr)),
                OP_JT => print!("jt         {:08x}", instr::d_op_26(instr)),
                OP_JF => print!("jf         {:08x}", instr::d_op_26(instr)),

                OP_CALL => print!("call       {}", instr::d_op_12(instr)),
                OP_RET => print!("ret        "),

                OP_POPVAL => print!("popval     {}", instr::d_op_12(instr)),

                _ => print!("???        "),
            }

            if let Some(comment) = self.comments.get(&(addr as u32)) {
                print!("      \t; {}", comment);
            }
            println!("");
        }

        if self.literals.len() > 0 {
            println!("================================================");
            println!("==== LITERALS");
            for (i, v) in self.literals.iter().enumerate() {
                println!("[{:5} ] {:?}", i, v);
            }
        }
        println!("================================================");
    }
}
