use std::rc::Rc;

use parser::*;
use parser::{ParserError, ParserErrorValue};

use lexer::TokenType;

pub struct Parser {
    traveler: Traveler,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(traveler: Traveler) -> Parser {
        Parser {
            traveler,
        }
    }

    pub fn parse(&mut self) -> ParserResult<Vec<Statement>> {
        let mut stack = Vec::new();
        while self.traveler.remaining() > 1 {
            stack.push(try!(self.statement()));
            self.traveler.next();
        }
        Ok(stack)
    }

    pub fn statement(&mut self) -> ParserResult<Statement> {
        match self.traveler.current().token_type {
            TokenType::Identifier => {
                let id = self.traveler.current_content().clone();

                self.traveler.next();

                if self.traveler.current_content() != ":=" {
                    self.traveler.prev();

                    return Ok(Statement::Expression(Box::new(try!(self.expression()))))
                }

                self.traveler.next();

                if self.traveler.current_content() == "\n" {
                    Ok(Statement::Definition { var: Rc::new(id), val: None })
                } else {
                    let value = try!(self.expression());
                    Ok(Statement::Definition { var: Rc::new(id), val: Some(Box::new(value)) })
                }
            },

            TokenType::EOL => {
                self.traveler.next();
                self.statement()
            },

            _ => Ok(Statement::Expression(Box::new(try!(self.expression())))),
        }
    }

    pub fn term(&mut self) -> ParserResult<Expression> {
        match self.traveler.current().token_type {
            TokenType::EOL => {
                self.traveler.next();
                match self.traveler.current().token_type {
                    TokenType::Block(_) => return Ok(Expression::Block(Box::new(try!(self.block())))),
                    TokenType::EOL      => return Ok(Expression::EOF),
                    _ => (),
                }
            },
            _ => (),
        }

        match self.traveler.current().token_type {
            TokenType::IntLiteral    => Ok(Expression::IntLiteral(self.traveler.current_content().parse::<i64>().unwrap())),
            TokenType::FloatLiteral  => Ok(Expression::FloatLiteral(self.traveler.current_content().parse::<f64>().unwrap())),
            TokenType::BoolLiteral   => Ok(Expression::BoolLiteral(self.traveler.current_content() == "true")),
            TokenType::StringLiteral => Ok(Expression::StringLiteral(self.traveler.current_content().clone())),
            TokenType::Identifier    => {
                let expr = self.traveler.current_content();

                self.traveler.next();

                match self.traveler.current().token_type {
                    TokenType::Operator => return self.operation(Expression::Identifier(expr)),
                    TokenType::Symbol => match self.traveler.current_content().as_str() {
                        "(" => return self.call(Expression::Identifier(expr)),
                        ":" => return self.function(expr),
                        _ => (),
                    },
                    TokenType::EOL => {
                        self.traveler.next();
                    },
                    _ => return Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected: {}", self.traveler.current_content()))),
                }

                Ok(Expression::Identifier(expr))
            },
            TokenType::Symbol => match self.traveler.current_content().as_str() {
                "(" => {
                    self.traveler.next();
                    let expr = try!(self.expression());
                    self.traveler.next();
                    self.traveler.expect_content(")");

                    self.traveler.next();

                    if self.traveler.current_content() == "(" {
                        return self.call(expr)
                    }

                    Ok(expr)
                },
                s => Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected symbol: {}", s))),
            },
            _ => Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected: {}", self.traveler.current_content()))),
        }
    }

    fn function(&mut self, name: String) -> ParserResult<Expression> {
        self.traveler.next(); // skip colon

        let mut params = Vec::new();

        while self.traveler.current_content() != "->" {
            match self.traveler.current().token_type {
                TokenType::Identifier => params.push(Rc::new(self.traveler.current_content())),
                TokenType::Symbol => match self.traveler.current_content().as_str() {
                    "," => { self.traveler.next(); },
                    s   => return Err(ParserError::new(&format!("unexpected symbol: {}", s))),
                },
                _ => return Err(ParserError::new(&format!("unexpected token: {}", self.traveler.current_content())))
            }
        }

        self.traveler.next(); // skip ->

        match self.traveler.current_content().as_str() {
            "\n" => {
                self.traveler.next();

                let lambda = Lambda::new(params, Box::new(try!(self.block())));

                Ok(Expression::Function(Function::new(Rc::new(name), Rc::new(lambda))))
            },

            s => Err(ParserError::new(&format!("unexpected: {}", s))),
        }
    }

    fn call(&mut self, expr: Expression) -> ParserResult<Expression> {
        self.traveler.next();

        let mut stack = vec![];

        while self.traveler.current_content() != ")" {
            if self.traveler.current_content() == "\n" {
                break
            }

            stack.push(try!(self.expression()));
            self.traveler.next();

            if self.traveler.current_content() == "," {
                self.traveler.next();
            }
        }

        self.traveler.next(); // skips ')'

        Ok(Expression::Call {
            func: Box::new(expr),
            args: Box::new(stack),
        })
    }

    fn block(&mut self) -> ParserResult<Vec<Statement>> {
        match self.traveler.current().token_type {
            TokenType::Block(ref v) => {
                let mut p = Parser::new(Traveler::new(v.clone()));
                Ok(try!(p.parse()))
            },
            _ => Err(ParserError::new_pos(self.traveler.current().position, &format!("expected block, found: {}", self.traveler.current_content()))),
        }
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        if self.traveler.current_content() == "\n" {
            self.traveler.next();
        }

        let expr = try!(self.term());

        self.traveler.next();
        if self.traveler.remaining() > 0 {
            if self.traveler.current().token_type == TokenType::Operator {
                return self.operation(expr)
            }
            self.traveler.prev();
        }
        Ok(expr)
    }

    fn operation(&mut self, expression: Expression) -> ParserResult<Expression> {
        let mut ex_stack = vec![expression];
        let mut op_stack: Vec<(Operand, u8)> = Vec::new();

        op_stack.push(operand(&self.traveler.current_content()).unwrap());
        self.traveler.next();

        if self.traveler.current_content() == "\n" {
            self.traveler.next();
        }

        ex_stack.push(try!(self.term()));

        let mut done = false;
        while ex_stack.len() > 1 {
            if !done && self.traveler.next() {
                if self.traveler.current().token_type != TokenType::Operator {
                    self.traveler.prev();
                    done = true;
                    continue
                }

                let (op, precedence) = operand(&self.traveler.current_content()).unwrap();

                if precedence >= op_stack.last().unwrap().1 {
                    let left  = ex_stack.pop().unwrap();
                    let right = ex_stack.pop().unwrap();

                    ex_stack.push(Expression::Operation {
                        right:  Box::new(left),
                        op:    op_stack.pop().unwrap().0,
                        left: Box::new(right)
                    });

                    self.traveler.next();

                    ex_stack.push(try!(self.term()));
                    op_stack.push((op, precedence));

                    continue
                }

                self.traveler.next();

                ex_stack.push(try!(self.term()));
                op_stack.push((op, precedence));
            }

            let left  = ex_stack.pop().unwrap();
            let right = ex_stack.pop().unwrap();

            ex_stack.push(Expression::Operation {
                right:  Box::new(left),
                op:    op_stack.pop().unwrap().0,
                left: Box::new(right)
            });
        }

        Ok(ex_stack.pop().unwrap())
    }
}