use std::rc::Rc;

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
    Plus,
    Minus,
    Equals,
    NEquals,
    Lt,
    Gt,
    LtEquals,
    GtEquals,
    And,
    Or,
}

pub fn operand(v: &str) -> Option<(Operand, u8)> {
    match v {
        "*"   => Some((Operand::Mul, 1)),
        "/"   => Some((Operand::Div, 1)),
        "%"   => Some((Operand::Mod, 1)),
        "^"   => Some((Operand::XOR, 1)),
        "+"   => Some((Operand::Plus, 2)),
        "-"   => Some((Operand::Minus, 2)),
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