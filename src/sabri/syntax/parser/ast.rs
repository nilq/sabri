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