use crate::types::token::*;
use crate::types::ast::*;

use std::{rc::Rc};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub id_counter: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
            id_counter: 0
        }
    }

    // --------------------- operators ---------------------

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name , id } => {
                    return Ok(
                        Expr::Assign { name, value: Box::new(value), id }
                    );
                },
                Expr::Get { object, name } => {
                    return Ok(
                        Expr::Set { object: object, name, value: Box::new(value) }
                    )
                }
                _ => {
                    return Err(parse_error(&equals, "Invalid assignment target.".to_string()));
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
            else if self.match_token(&[TokenType::Dot]) {
                expr = Expr::Get{
                    object: Box::new(expr),
                    name: self.consume(
                        TokenType::Identifier,
                        "Expect property name after '.'.".to_string()
                    )?.clone()
                }
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
                    return Err(parse_error(self.peek(), "Can't have more than 255 arguments.".to_string()));
                }

                arguments.push(self.expression()?);
            
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.".to_string())?;

        Ok(Expr::Call { callee: Box::new(callee), paren: paren.clone(), args: arguments })
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name".to_string())?.clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else { None };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration".to_string())?;
        Ok(Stmt::Var { name: name, initializer: initializer })
    }

    fn class_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected class name.".to_string())?.clone();

        let mut superclass = None;
        if self.match_token(&[TokenType::Less]) {
            self.consume(TokenType::Identifier, "Expect superclass name.".to_string())?;
            self.id_counter +=1 ;
            superclass = Some(Expr::Variable { name: self.previous().clone(), id: self.id_counter - 1 })
        }

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.".to_string())?;

        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.".to_string())?;

        Ok(Stmt::Class { name, methods, superclass })
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::False]) { return Ok(Expr::Literal { value: Literal::Bool(false) }); }
        if self.match_token(&[TokenType::True])  { return Ok(Expr::Literal { value: Literal::Bool(true)  }); }
        if self.match_token(&[TokenType::Nil])   { return Ok(Expr::Literal { value: Literal::Nil         }); }
        
        if self.match_token(&[TokenType::Super]) {
            let keyword = self.previous().clone();
            self.consume(TokenType::Dot, "Expect '.' after 'super'.".to_string())?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.".to_string())?.clone();
            self.id_counter += 1;
            return Ok(Expr::Super { keyword, method, id: self.id_counter - 1 })
        }

        if self.match_token(&[TokenType::This]) {
            self.id_counter += 1;
            return Ok(Expr::This { keyword: self.previous().clone(), id: self.id_counter - 1 });
        }

        if self.match_token(&[TokenType::Num, TokenType::Str]) {
            return Ok(Expr::Literal { value: self.previous().literal.clone().expect("Missing literal in parser.rs::primary()") });
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.".to_string())?;
            return Ok(Expr::Grouping { expression: Box::new(expr) });
        }

        if self.match_token(&[TokenType::Identifier]) {
            self.id_counter += 1;
            return Ok(Expr::Variable { name: self.previous().clone(), id: self.id_counter - 1 })
        }

        Err(parse_error(self.peek(), "Expect expression.".to_string()))
    }

    // ------------ error recovery ------------

    fn consume(&mut self, ty: TokenType, msg: String) -> Result<&Token, ParseError> {
        if self.check(ty) { return Ok(self.advance()); }

        Err(parse_error(self.peek(), msg))
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
        if self.match_token(&[TokenType::Fun]) {
            return self.function("function");
        }

        if self.match_token(&[TokenType::Class]) {
            return self.class_declaration();
        }

        if self.match_token(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block { statements: self.block_statement()? });
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
        self.consume(TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Print(Box::new(expr)))
    }

    fn block_statement(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> =  Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.".to_string())?;
        
        Ok(statements)
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.".to_string())?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.".to_string())?;

        Ok(Stmt::If {
            condition,
            then_branch:
                Box::new(self.statement()?),
            else_branch:
                if self.match_token(&[TokenType::Else]) { Some(Box::new(self.statement()?)) }
                else { None }
        })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen ,"Expect '(' after 'while'.".to_string())?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen ,"Expect ')' after 'while'.".to_string())?;

        Ok(Stmt::While { condition, body: Box::new(self.statement()?) })
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let keyword = self.previous().clone();
        let mut value: Option<Expr> = None;

        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression()?);
        }
        
        self.consume(TokenType::Semicolon, "Expect ';' after return value.".to_string())?;
        Ok(Stmt::Return { keyword, value })
    }

    fn function(&mut self, kind: &str) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, format!("Expect {kind} name."))?.clone();
        self.consume(TokenType::LeftParen, format!("Expect '(' after {kind} name."))?;
        
        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(parse_error(&name,"Can't have more than 255 parameters.".to_string()));
                }

                parameters.push(
                    self.consume(TokenType::Identifier, "Expect parameter name.".to_string())?.clone()
                );

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.".to_string())?;
        self.consume(TokenType::LeftBrace, format!("Expect '{{' before {kind} body."))?;

        Ok(Stmt::Function (
            Rc::new(
                FuncDecl { name, params: parameters, body: self.block_statement()? }
            )
        ))
    }

    fn desugaring_for(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.".to_string())?;

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
        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.".to_string())?;

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expected ')' after for clauses.".to_string())?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block { statements: vec![body, Stmt::Expression(Box::new(increment))] }
        }

        body = Stmt::While {
            condition: condition.unwrap_or_else(|| Expr::Literal { value: Literal::Bool(true) }),
            body: Box::new(body)
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block { statements: vec![initializer, body] }
        }

        Ok(body)
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Expression(Box::new(expr)))
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