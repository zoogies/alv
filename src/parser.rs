use std::fmt::Error;

use crate::{lexer::{TokenType::Semicolon, *}, log::alv_error, parser::{Expr::Assign, Stmt::{ExpressionStmt, PrintStmt}}};

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>
    },
    Grouping {
        expression: Box<Expr>
    },
    Literal {
        value: Literal
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>
    },
    Variable {
        name: Token
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: &'static str,
}

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum Stmt {
    ExpressionStmt(Box<Expr>),
    PrintStmt(Box<Expr>),
    VarStmt{name: Token, initializer: Option<Box<Expr>>},
    BlockStmt{statements: Vec<Stmt>},
    IfStmt{condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    WhileStmt{condition: Expr, body: Box<Stmt>}
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
        }
    }

    // --------------------- operators ---------------------

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => {
                    return Ok(
                        Expr::Assign { name, value: Box::new(value) }
                    );
                }
                _ => {
                    return Err(self.error(&equals, "Invalid assignment target."));
                }
            }
        }

        Ok(expr)
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
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

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.and()?)
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.equality()?)
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
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

    fn term(&mut self) -> ParseResult<Expr> {
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

    fn factor(&mut self) -> ParseResult<Expr> {
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

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[
            TokenType::Bang,
            TokenType::Minus
        ]) {
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                right: Box::new(self.unary()?),
            })
        }

        self.call()
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            }
            else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(self.error(self.peek(), "Can't have more than 255 arguments."));
                }

                arguments.push(self.expression()?);
            
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call { callee: Box::new(callee), paren: paren.clone(), args: arguments })
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?.clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else { None };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration")?;
        Ok(Stmt::VarStmt { name: name, initializer: initializer })
    }

    fn primary(&mut self) -> ParseResult<Expr> {
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

    fn error(&self, token: &Token, msg: &'static str) -> ParseError {
        if token.token_type == TokenType::EndFile {
            alv_error!("[line {}] at end: {}", token.line + 1, msg);
        } else {
            alv_error!("[line {}] at '{}': {}", token.line + 1, token.lexeme, msg);
        }

        ParseError {
            token: token.clone(),
            message: msg,
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &'static str) -> Result<&Token, ParseError> {
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
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1 }
        self.previous()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::EndFile
    }

    fn _idx(&self, delta: isize) -> &Token {
        let idx = self.current.checked_add_signed(delta).unwrap_or_else(|| {
            panic!("under/overflow in parser.rs::_idx: current={} delta={}", self.current, delta)
        });

        self.tokens.get(idx).unwrap_or_else(|| {
            panic!("parser index out of bounds: idx={} current={} delta={} len={}",idx,self.current,delta,self.tokens.len());
        })
    }

    fn peek(&self) -> &Token {
        self._idx(0)
    }

    fn previous(&self) -> &Token {
        self._idx(-1)
    }

    // --------------------- actually parse ---------------------
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ()> {
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

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return self.block_statement();
        }

        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_token( &[TokenType::For]) {
            return self.desugaring_for();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(PrintStmt(Box::new(expr)))
    }

    fn block_statement(&mut self) -> ParseResult<Stmt> {
        let mut statements: Vec<Stmt> =  Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        
        Ok(Stmt::BlockStmt { statements })
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        Ok(Stmt::IfStmt {
            condition,
            then_branch:
                Box::new(self.statement()?),
            else_branch:
                if self.match_token(&[TokenType::Else]) { Some(Box::new(self.statement()?)) }
                else { None }
        })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen ,"Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen ,"Expect ')' after 'while'.")?;

        Ok(Stmt::WhileStmt { condition, body: Box::new(self.statement()?) })
    }

    fn desugaring_for(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;

        let initializer: Option<Stmt>;
        if self.match_token(&[TokenType::Semicolon]) {
            initializer = None;
        }
        else if self.match_token(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        }
        else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expected ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::BlockStmt { statements: vec![body, Stmt::ExpressionStmt(Box::new(increment))] }
        }

        body = Stmt::WhileStmt {
            condition: condition.unwrap_or_else(|| Expr::Literal { value: Literal::Bool(true) }),
            body: Box::new(body)
        };

        if let Some(initializer) = initializer {
            body = Stmt::BlockStmt { statements: vec![initializer, body] }
        }

        Ok(body)
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
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