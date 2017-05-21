use parser::{Traveler, Expression, Statement, Operand, ParserResult, operand};
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
            _ => Err(ParserError::new(self.traveler.current().position, &format!("unexpected: {}", self.traveler.current_content()))),
        }
    }

    fn block(&mut self) -> ParserResult<Vec<Statement>> {
        match self.traveler.current().token_type {
            TokenType::Block(ref v) => {
                let mut p = Parser::new(Traveler::new(v.clone()));
                Ok(try!(p.parse()))
            },
            _ => Err(ParserError::new(self.traveler.current().position, &format!("expected block, found: {}", self.traveler.current_content()))),
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