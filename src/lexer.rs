use crate::log::alv_error;
use crate::log::alv_log;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    LeftBrace, RightBrace,
    LeftParen, RightParen,
    Plus, Minus, Star, Slash,
    Comma, Dot, Semicolon,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, Str, Num,

    And, Class, Else, False, True,
    Fun, For, If, Nil, Print, Or,
    Return, Super, This, Var, While,

    EndFile
}

impl TokenType {
    fn from_ident(text: &str) -> Self {
        match text {
            "and" => Self::And,
            "class" => Self::Class,
            "else" => Self::Else,
            "false" => Self::False,
            "fun" => Self::Fun,
            "for" => Self::For,
            "if" => Self::If,
            "nil" => Self::Nil,
            "or" => Self::Or,
            "print" => Self::Print,
            "return" => Self::Return,
            "super" => Self::Super,
            "this" => Self::This,
            "true" => Self::True,
            "var" => Self::Var,
            "while" => Self::While,
            _ => Self::Identifier,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal<'p> {
    String(&'p str),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub struct Token<'p> {
    pub token_type: TokenType,
    pub lexeme: &'p str,
    pub line: usize,
    pub literal: Option<Literal<'p>>,
}

impl<'p> Token<'p> {
    fn new(token_type: TokenType, lexeme: &'p str, line: usize, lit: Option<Literal<'p>>) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            literal: lit,
        }
    }
}

pub struct Lexer<'p> {
    input: &'p str, 
    tokens: Vec<Token<'p>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'p> Lexer<'p> {
    pub fn new(input: &'p str) -> Self {
        Self {
            // why do these require manual annotation?
            input: input,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.input.as_bytes()[self.current - 1] as char
    }

    fn match_advance(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false }

        match self.input.chars().nth(self.current) {
            None => {
                alv_error!("Could not retrieve character: {} from nth({})", expected, self.current);
                return false;
            }
            Some(v) => {
                if v != expected {
                    return false;
                }
                
                self.current+=1;
                return true;
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(
            Token::new(
                token_type, 
                &self.input[self.start..self.current], 
                self.line,
                None
            )
        );
    }

    fn add_token_literal(&mut self, token_type: TokenType, lit: Literal<'p>) {
        self.tokens.push(
            Token::new(
                token_type, 
                &self.input[self.start..self.current], 
                self.line,
                Some(lit)
            )
        );
    }

    fn peek(&self) -> char {
        if self.is_at_end() {return '\0'};
        match self.input.chars().nth(self.current) {
            None => {
                alv_error!("Could not peek character at nth({})", self.current);
                return '\0';
            }
            Some(v) => {
                return v;
            }
        }
    }

    fn peek_next(&self) -> char {
        if self.current+1 >= self.input.len() { return '\0'; }
        match self.input.chars().nth(self.current+1) {
            None => {
                alv_error!("Could not peek next character at nth({})", self.current);
                return '\0';
            }
            Some(c) => {
                return c;
            }
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {self.line+=1; self.advance(); }
            self.advance();
        }
        
        if self.is_at_end() {
            alv_error!("Unterminated string.\n");
            return;
        }

        // closing "
        self.advance();

        self.add_token_literal(TokenType::Str, Literal::String(&self.input[self.start+1..self.current-1]));
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(&self, c:char ) -> bool {
        c.is_alphabetic()
    }

    fn is_alphanumeric(&self, c:char) -> bool {
        c.is_alphanumeric()
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) { self.advance(); }

        // look for fractional component
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {self.advance();}
        }

        self.add_token_literal(TokenType::Num, Literal::Number(
            self.input[self.start..self.current].parse::<f64>().expect("Not a valid number")
        ))
    }

    fn identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {self.advance();}

        self.add_token(TokenType::from_ident(&self.input[self.start..self.current]));
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();

        match c {
            // single char tokens:
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '*' => self.add_token(TokenType::Star),
            ';' => self.add_token(TokenType::Semicolon),
            '.' => self.add_token(TokenType::Dot),
            ',' => self.add_token(TokenType::Comma),

            // tokens that depend on mutli-char sequences:
            '!' => if self.match_advance('=') {self.add_token(TokenType::BangEqual)} else {self.add_token(TokenType::Bang)},
            '=' => if self.match_advance('=') {self.add_token(TokenType::EqualEqual)} else {self.add_token(TokenType::Equal)},
            '>' => if self.match_advance('=') {self.add_token(TokenType::GreaterEqual)} else {self.add_token(TokenType::Greater)},
            '<' => if self.match_advance('=') {self.add_token(TokenType::LessEqual)} else {self.add_token(TokenType::Less)},
            
            // special case, comments use "//" and require ignoring the whole rest of the line
            '/' => if self.match_advance('/') { 
                while(self.peek() != '\n') && !self.is_at_end() {
                    self.advance();
                }}
                else {
                    self.add_token(TokenType::Slash);
                },
            
            // fillers
            ' '  |
            '\r' |
            '\t' 
            => {}

            '\n' => {self.line+=1;}

            // literals
            '"' => {self.string();}
            _ if self.is_digit(c) => { self.number(); }

            // reserved words and identifiers
            _ if self.is_alpha(c) => {
                self.identifier();
            }

            _ => {            
                alv_error!("Unexpected token: {}", c);
            }
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token<'p>> {
        while !self.is_at_end() {
            alv_log!("self.current: {} self.input[self.current]: {:?}\n",self.current,self.input.chars().nth(self.current));

            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token { token_type: TokenType::EndFile, lexeme: "", line: self.line, literal: None });
        
        self.tokens
    }

}