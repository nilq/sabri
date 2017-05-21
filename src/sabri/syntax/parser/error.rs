use lexer::TokenPosition;

use std::fmt;

#[derive(Debug)]
pub enum ParserErrorValue {
    Constant(String),
}

#[derive(Debug)]
pub struct ParserError {
    value:    ParserErrorValue,
    position: TokenPosition,
}

impl ParserError {
    pub fn new(position: TokenPosition, value: &str) -> ParserError {
        ParserError {
            value: ParserErrorValue::Constant(value.to_owned()),
            position,
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            ParserErrorValue::Constant(ref s) => write!(f, "{}: {}", self.position, s),
        }
    }
}