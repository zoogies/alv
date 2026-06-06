use crate::{lexer::*, log::alv_error};

// TODO: had to infect this with <'p> because of lexeme string slice reference...
pub enum Expr<'p> {
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

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p Vec<Token<'p>>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    // --------------------- operators ---------------------

    fn expression(&mut self) -> ParseResult<'p, Expr<'p>> {
        self.equality()
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

        Err(self.error(self.peek(), "Expect expression."))
    }

    // ------------ error recovery ------------

    fn error(&self, token: &'p Token<'p>, msg: &'static str) -> ParseError<'p> {
        if token.token_type == TokenType::EndFile {
            alv_error!("[line {}] at end: {}", token.line, msg);
        } else {
            alv_error!("[line {}] at '{}': {}", token.line, token.lexeme, msg);
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
    pub fn parse(&mut self) -> Option<Expr<'p>> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => {
                self.synchronize();
                None
            }
        }
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

// --------------------- printer ---------------------

pub struct AstPrinter;

impl AstPrinter {
    pub fn print<'s>(&self, expr: &Expr<'s>) -> String {
        self.print_expr(expr)
    }

    fn print_expr<'s>(&self, expr: &Expr<'s>) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right
            } => {
                self.parenthesize(
                    operator.lexeme,
                    &[left.as_ref(), right.as_ref()],
                )
            }

            Expr::Grouping { expression} => {
                self.parenthesize(
                    "group",
                    &[expression.as_ref()],
                )
            }

            Expr::Literal { value } => {
                self.print_literal(value)
            }

            Expr::Unary { operator, right } => {
                self.parenthesize(
                    operator.lexeme,
                    &[right.as_ref()],
                )
            }
        }
    }

    fn print_literal<'s>(&self, value: &Literal<'s>) -> String {
        match value {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => s.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Nil => "nil".to_string()
        }
    }

    fn parenthesize<'s>(&self, name: &str, exprs: &[&Expr<'s>]) -> String {
        let mut out = String::new();

        out.push('(');
        out.push_str(name);

        for expr in exprs {
            out.push(' ');
            out.push_str(&self.print_expr(expr));
        }

        out.push(')');

        out
    }
}
