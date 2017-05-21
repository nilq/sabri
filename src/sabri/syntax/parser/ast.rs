use std::rc::Rc;
use std::fmt;

use parser::bytecode::Program;
use parser::{ParserResult, ParserError};

use sabri::SymTab;
use sabri::Value;

#[derive(Debug, Clone)]
pub enum Expression {
    Block(Box<Vec<Statement>>),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),

    Call {
        func: Box<Expression>,
        args: Box<Vec<Expression>>,
    },

    Operation {
        left: Box<Expression>,
        op: Operand,
        right: Box<Expression>,
    },

    EOF,
}

impl Expression {
    pub fn compile(&self, sym: &Rc<SymTab>, program: &mut Program) -> ParserResult<()> {
        match *self {
            Expression::IntLiteral(ref n) => {
                program.add_comment(&format!("{}", *n));

                let index = program.add_literal(Value::Number(*n as f64));
                program.emit_pushlit(index);
            },

            Expression::FloatLiteral(ref n) => {
                program.add_comment(&format!("{}", *n));

                let index = program.add_literal(Value::Number(*n));
                program.emit_pushlit(index);
            },

            Expression::StringLiteral(ref n) => {
                program.add_comment(&format!("{}", *n));

                let index = program.add_literal(Value::Str(Rc::new(n.clone())));
                program.emit_pushlit(index);
            },

            Expression::Identifier(ref id) => match sym.get_name(&*id) {
                Some((i, env_index)) => {
                    program.add_comment(&*id);
                    program.emit_getvar(i as u16, env_index as u16)
                },
                None => return Err(ParserError::new(&format!("undeclared identifier: {}", id)))
            },

            Expression::Call {ref func, ref args} => {
                try!(func.compile(sym, program));

                for a in &**args {
                    try!(a.compile(sym, program))
                }

                program.emit_call(args.len() as u16);
                return Ok(())
            },

            Expression::Block(ref s) => {
                let mut cur_sym = sym.clone();
                let mut env_vars = 0;
                let mut fix_newenv_addr = 0;

                for statement in &**s {
                    match statement {
                        &Statement::Definition { ref var, ref val } => {
                            match *val {
                                Some(ref e) => try!(e.compile(&cur_sym, program)),
                                None => {
                                    let index = program.add_literal(Value::Null);
                                    program.add_comment("null");
                                    program.emit_pushlit(index);
                                },
                            }

                            if env_vars == 0{
                                cur_sym = Rc::new(SymTab::new(cur_sym.clone(), &[var.clone()]));
                                program.increment_env_level(1);
                                
                                fix_newenv_addr = program.addr();
                                
                                program.add_comment(&format!("var {} = ...", &*var));
                                program.emit_newenv(1, 1);
                            }
                        },
                        _ => try!(statement.compile(&cur_sym, program)),
                    }
                }
            },

            Expression::Operation {ref left, ref op, ref right} => match op {
                &Operand::Assign => try!(self.compile_assignment(&*left, &*right, sym, program)),
                o => match o {
                    &Operand::Add |
                    &Operand::Sub |
                    &Operand::Mul |
                    &Operand::Div => {
                        try!(left.compile(sym, program));
                        try!(right.compile(sym, program));

                        match op {
                            &Operand::Add => program.emit_add(),
                            &Operand::Sub => program.emit_sub(),
                            &Operand::Mul => program.emit_mul(),
                            &Operand::Div => program.emit_div(),
                            _ => return Err(ParserError::new(&format!("unhandled operator: {}", op))),
                        }
                    },
                    op => {
                        let op = &format!("{}", op);

                        let (vi, ei) = match sym.get_name(op) {
                            Some((vi, ei)) => (vi, ei),
                            None => return Err(ParserError::new(&format!("operator doesn't exist: '{}'", op)))
                        };
                        program.add_comment(op);
                        program.emit_getvar(vi as u16, ei as u16);

                        try!(left.compile(sym, program));
                        try!(right.compile(sym, program));
                
                        program.emit_call(2);
                    },
                }
            },

            Expression::EOF => return Ok(()),

            _ => return Err(ParserError::new("accessing unimplemented codegen")),
        }
        Ok(())
    }

    pub fn compile_assignment(&self, l: &Expression, value: &Expression, sym: &Rc<SymTab>, program: &mut Program) -> ParserResult<()> {
        match *l {
            Expression::Identifier(ref s) => match sym.get_name(&*s) {
                Some((i, env_index)) => {
                    try!(value.compile(sym, program));

                    program.add_comment(&format!("{} = ..", &*s));
                    program.emit_setvar(i as u16, env_index as u16);

                    return Ok(())
                },

                None => return Err(ParserError::new(&format!("can't assign undefined variable: {}", s))),
            },

            _ => Err(ParserError::new("can't assign invalid target")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Box<Expression>),
    Definition {
        var: Rc<String>,
        val: Option<Box<Expression>>,
    },
    Assignment(Box<Expression>, Box<Expression>),
}

impl Statement {
    pub fn compile(&self, sym: &Rc<SymTab>, program: &mut Program) -> ParserResult<()> {
        match *self {
            Statement::Expression(ref e) => e.compile(sym, program),
            _ => Err(ParserError::new("unimplemented statement bytecode"))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operand {
    Mul,
    Div,
    Mod,
    XOR,
    Add,
    Sub,
    Equals,
    NEquals,
    Lt,
    Gt,
    LtEquals,
    GtEquals,
    And,
    Or,
    Assign,
    Dot,
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::Add => write!(f, "+"),
            Operand::Sub => write!(f, "-"),
            Operand::Mul => write!(f, "*"),
            Operand::Div => write!(f, "/"),
            Operand::Mod => write!(f, "%"),
            Operand::XOR => write!(f, "^"),
            Operand::Equals => write!(f, "=="),
            Operand::NEquals => write!(f, "!="),
            Operand::Lt => write!(f, "<"),
            Operand::LtEquals => write!(f, "<="),
            Operand::Gt => write!(f, ">"),
            Operand::GtEquals => write!(f, ">="),
            Operand::And => write!(f, "and"),
            Operand::Assign => write!(f, "="),
            Operand::Dot => write!(f, "."),
            Operand::Or => write!(f, "or"),
        }
    }
}

pub fn operand(v: &str) -> Option<(Operand, u8)> {
    match v {
        "*"   => Some((Operand::Mul, 1)),
        "/"   => Some((Operand::Div, 1)),
        "%"   => Some((Operand::Mod, 1)),
        "^"   => Some((Operand::XOR, 1)),
        "+"   => Some((Operand::Add, 2)),
        "-"   => Some((Operand::Sub, 2)),
        "=="  => Some((Operand::Equals, 3)),
        "!="  => Some((Operand::NEquals, 3)),
        "<"   => Some((Operand::Lt, 4)),
        ">"   => Some((Operand::Gt, 4)),
        "<="  => Some((Operand::LtEquals, 4)),
        ">="  => Some((Operand::GtEquals, 4)),
        "and" => Some((Operand::And, 4)),
        "or"  => Some((Operand::Or, 4)),
        "."   => Some((Operand::Dot, 5)),
        "="   => Some((Operand::Assign, 6)),
        _ => None,
    }
}