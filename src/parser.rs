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

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p Vec<Token<'p>>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    // --------------------- operators ---------------------

    fn expression(&mut self) -> Expr<'p> {
        self.equality()
    }

    fn equality(&mut self) -> Expr<'p> {
        let mut expr: Expr = self.comparison();
    
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.comparison();
            
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right)
            }
        }

        expr
    }

    fn comparison(&mut self) -> Expr<'p> {
        let mut expr = self.term();

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ]) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.term()),
            }
        }

        expr
    }

    fn term(&mut self) -> Expr<'p> {
        let mut expr = self.factor();

        while self.match_token(&[
            TokenType::Minus,
            TokenType::Plus,
        ]) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.factor()),
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr<'p> {
        let mut expr = self.unary();

        while self.match_token(&[
            TokenType::Slash,
            TokenType::Star,
        ]) {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: self.previous().clone(),
                right: Box::new(self.unary()),
            }
        }

        expr
    }

    fn unary(&mut self) -> Expr<'p> {
        if self.match_token(&[
            TokenType::Bang,
            TokenType::Minus
        ]) {
            return Expr::Unary {
                operator: self.previous().clone(),
                right: Box::new(self.unary()),
            }
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr<'p> {
        if self.match_token(&[TokenType::False]) { return Expr::Literal { value: Literal::Bool(false) }; }
        if self.match_token(&[TokenType::True]) { return Expr::Literal { value: Literal::Bool(true) }; }
        if self.match_token(&[TokenType::Nil]) { return Expr::Literal { value: Literal::Nil }; }

        if self.match_token(&[TokenType::Num, TokenType::Str]) {
            return Expr::Literal { value: self.previous().literal.expect("Missing literal in parser.rs::primary()") };
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping { expression: Box::new(expr) };
        }

        panic!("Bad case in parser.rs::primary(): fell through");
    }

    // ------------ error recovery ------------

    fn consume(&mut self, ty: TokenType, msg: &str) -> &Token { // TODO: is ref to token?
        if self.check(ty) { return self.advance(); }

        
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

    fn check(&self, t: TokenType) -> bool {
        if self.is_at_end() { return false; }
        self.peek().token_type == t // TODO check literal "type" not classified
    }
    
    fn advance(&mut self) -> &Token<'p> {
        if !self.is_at_end() { self.current += 1 }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EndFile
    }

    fn _idx(&self, delta: isize) -> &Token<'p> {
        let idx = self.current.checked_add_signed(delta).unwrap_or_else(|| {
            panic!("under/overflow in parser.rs::_idx: current={} delta={}", self.current, delta)
        });

        self.tokens.get(idx).unwrap_or_else(|| {
            panic!("parser index out of bounds: idx={} current={} delta={} len={}",idx,self.current,delta,self.tokens.len());
        })
    }

    fn peek(&self) -> &Token<'p> {
        self._idx(0)
    }

    fn previous(&self) -> &Token<'p> {
        self._idx(-1)
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
