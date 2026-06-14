use std::fmt::Error;

use crate::{lexer::*, log::alv_error, parser::{Expr::Assign, Stmt::{ExpressionStmt, PrintStmt}}};

// TODO: had to infect this with <'p> because of lexeme string slice reference...
pub enum Expr<'p> {
    Assign {
        name: Token<'p>,
        value: Box<Expr<'p>>
    },
    Binary {
        left: Box<Expr<'p>>,
        operator: Token<'p>,
        right: Box<Expr<'p>>
    },
    Grouping {
        expression: Box<Expr<'p>>
    },
    Literal {
        value: Literal<'p>
    },
    Unary {
        operator: Token<'p>,
        right: Box<Expr<'p>>
    },
    Variable {
        name: Token<'p>
    }
}

pub struct Parser<'p> {
    tokens: &'p Vec<Token<'p>>,
    current: usize,
}

#[derive(Debug, Clone)]
pub struct ParseError<'p> {
    pub token: &'p Token<'p>,
    pub message: &'static str,
}

type ParseResult<'p, T> = Result<T, ParseError<'p>>;

pub enum Stmt<'p> {
    ExpressionStmt(Box<Expr<'p>>),
    PrintStmt(Box<Expr<'p>>),
    VarStmt{name: Token<'p>, initializer: Option<Box<Expr<'p>>>},
}

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p Vec<Token<'p>>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    // --------------------- operators ---------------------

    fn assignment(&mut self) -> ParseResult<'p, Expr<'p>> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => {
                    return Ok(
                        Expr::Assign { name, value: Box::new(value) }
                    );
                }
                _ => {
                    return Err(self.error(equals, "Invalid assignment target."));
                }
            }
        }

        Ok(expr)
    }

    fn expression(&mut self) -> ParseResult<'p, Expr<'p>> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Expr<'p>, ParseError<'p>> {
        let mut expr: Expr = self.comparison()?;
    
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.comparison()?;
            
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right)
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<'p, Expr<'p>> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ]) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.term()?),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<'p, Expr<'p>> {
        let mut expr = self.factor()?;

        while self.match_token(&[
            TokenType::Minus,
            TokenType::Plus,
        ]) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.factor()?),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<'p, Expr<'p>> {
        let mut expr = self.unary()?;

        while self.match_token(&[
            TokenType::Slash,
            TokenType::Star,
        ]) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.unary()?),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<'p, Expr<'p>> {
        if self.match_token(&[
            TokenType::Bang,
            TokenType::Minus
        ]) {
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                right: Box::new(self.unary()?),
            })
        }

        self.primary()
    }

    fn var_declaration(&mut self) -> ParseResult<'p, Stmt<'p>> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?.clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else { None };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration")?;
        Ok(Stmt::VarStmt { name: name, initializer: initializer })
    }

    fn primary(&mut self) -> ParseResult<'p, Expr<'p>> {
        if self.match_token(&[TokenType::False]) { return Ok(Expr::Literal { value: Literal::Bool(false) }); }
        if self.match_token(&[TokenType::True])  { return Ok(Expr::Literal { value: Literal::Bool(true)  }); }
        if self.match_token(&[TokenType::Nil])   { return Ok(Expr::Literal { value: Literal::Nil         }); }

        if self.match_token(&[TokenType::Num, TokenType::Str]) {
            return Ok(Expr::Literal { value: self.previous().literal.clone().expect("Missing literal in parser.rs::primary()") });
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping { expression: Box::new(expr) });
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable { name: self.previous().clone() })
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    // ------------ error recovery ------------

    fn error(&self, token: &'p Token<'p>, msg: &'static str) -> ParseError<'p> {
        if token.token_type == TokenType::EndFile {
            alv_error!("[line {}] at end: {}", token.line + 1, msg);
        } else {
            alv_error!("[line {}] at '{}': {}", token.line + 1, token.lexeme, msg);
        }

        ParseError {
            token,
            message: msg,
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &'static str) -> Result<&'p Token<'p>, ParseError<'p>> {
        if self.check(ty) { return Ok(self.advance()); }

        Err(self.error(self.peek(), msg))
    }

    // ------------ token ops ------------

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for &t in types {
            if self.check(t) {
                self.advance();
                return true
            }
        }
        false
    }

    fn check(&mut self, t: TokenType) -> bool {
        if self.is_at_end() { return false; }
        self.peek().token_type == t // TODO check literal "type" not classified
    }
    
    fn advance(&mut self) -> &'p Token<'p> {
        if !self.is_at_end() { self.current += 1 }
        self.previous()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::EndFile
    }

    fn _idx(&self, delta: isize) -> &'p Token<'p> {
        let idx = self.current.checked_add_signed(delta).unwrap_or_else(|| {
            panic!("under/overflow in parser.rs::_idx: current={} delta={}", self.current, delta)
        });

        self.tokens.get(idx).unwrap_or_else(|| {
            panic!("parser index out of bounds: idx={} current={} delta={} len={}",idx,self.current,delta,self.tokens.len());
        })
    }

    fn peek(&self) -> &'p Token<'p> {
        self._idx(0)
    }

    fn previous(&self) -> &'p Token<'p> {
        self._idx(-1)
    }

    // --------------------- actually parse ---------------------
    pub fn parse(&mut self) -> Result<Vec<Stmt<'p>>, ()> {
        let mut v: Vec<Stmt> = Vec::new();
        let mut had_error = false;
        while !self.is_at_end() {
            match self.declaration() {
                Ok(s) => v.push(s),
                Err(_e) => { had_error = true; self.synchronize(); },
            }
        }

        if had_error { return Err(()); }
        
        Ok(v)
    }

    fn declaration(&mut self) -> ParseResult<'p, Stmt<'p>> {
        if self.match_token(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<'p, Stmt<'p>> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> ParseResult<'p, Stmt<'p>> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(PrintStmt(Box::new(expr)))
    }

    fn expression_statement(&mut self) -> ParseResult<'p, Stmt<'p>> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(ExpressionStmt(Box::new(expr)))
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {return;}
            match self.peek().token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var |
                TokenType::For | TokenType::If | TokenType::While |
                TokenType::Print | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

}