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

            Expression::Operation {ref left, ref op, ref right} => {
                match op {
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
                    _ => return Err(ParserError::new(&format!("unimplemented operator: {}", op))),
                }
            },

            _ => return Err(ParserError::new("accessing unimplemented codegen")),
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Box<Expression>),
    Definition(String, Box<Expression>),
    Assignment(Box<Expression>, Box<Expression>),
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
        _ => None,
    }
}