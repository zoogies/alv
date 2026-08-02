#[derive(Clone, Copy)]
pub enum TokenType {
    LeftParen, RightParen,
    LeftBrace, RightBrace,
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

    EndFile, Error
}

pub struct Token<'src> {
    slice:  &'src str,
    ty:     TokenType,
    line:   usize,
}

impl<'src> Token<'src> {
    pub fn error(str: &'src str, line: usize) -> Token {
        Self {
            slice: str,
            ty: TokenType::Error,
            line,
        }
    }
}

pub struct Scanner<'src> {
    src: &'src str,
    line: usize,
    start: usize,
    current: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            line: 1,
            start: 0,
            current: 0,
        }
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() { return b'\0'; }
        self.src.as_bytes()[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn make_token(&self, ty: TokenType) -> Token {
        Token { slice: &self.src[self.start..self.current], ty: ty, line: self.line }
    }

    fn advance(&mut self) -> u8 {
        let r = self.peek();
        self.current += 1;
        r
    }

    fn match_byte(&mut self, expected: u8) -> bool {
        if self.is_at_end() { return false; }

        if self.peek() != expected { return false; }

        self.current += 1;
        true
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.src.len() { return b'\0'; }
        self.src.as_bytes()[self.current + 1]
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                b' '  |
                b'\r' |
                b'\t' => {
                    self.advance();
                },
                b'\n' => {
                    self.line += 1;
                    self.advance();
                },
                b'/' => {
                    if self.peek_next() == b'/' {
                        while self.peek() != b'\n' && !self.is_at_end() { self.advance(); }
                    }
                    else {
                        return;
                    }
                }
                _ => { return; }
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' { self.line += 1; }
            self.advance();
        }

        if self.is_at_end() { return Token::error("Unterminated string.", self.line); }

        self.advance();
        self.make_token(TokenType::Str)
    }

    fn is_digit(&self, char: &u8) -> bool {
        char.is_ascii_digit()
    }

    fn number(&mut self) -> Token {
        while self.is_digit(&self.peek()) { self.advance(); }

        if self.peek() == b'.' && self.is_digit(&self.peek_next()) {
            self.advance();

            while self.is_digit(&self.peek()) { self.advance(); }
        }

        self.make_token(TokenType::Num)
    }

    fn is_alpha(&self, byte: &u8) -> bool {
        byte.is_ascii_alphabetic() || *byte == b'_'
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, ty: TokenType ) -> TokenType {
        if self.current - self.start == start + length && 
            &self.src[self.start + start..self.current] == rest
        {
            return ty;
        }

        TokenType::Identifier
    }

    // NOTE: technically idiomatic rust doesn't need the trie:
    //       match &self.src[self.start..self.current] { "and" => And, "class" => Class, ... }
    fn identifier_type(&self) -> TokenType {
        match self.src.as_bytes()[self.start] {
            b'a' => return self.check_keyword(1,2,"nd",TokenType::And),
            b'c' => return self.check_keyword(1,4,"lass",TokenType::Class),
            b'e' => return self.check_keyword(1,3,"lse",TokenType::Else),
            b'f' => {
                if self.current - self.start > 1 {
                    match self.src.as_bytes()[self.start + 1] {
                        b'a' => return self.check_keyword(2, 3, "lse", TokenType::False),
                        b'o' => return self.check_keyword(2, 1, "r", TokenType::For),
                        b'u' => return self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => { return TokenType::Identifier }
                    }
                }
                return TokenType::Identifier;
            },
            b'i' => return self.check_keyword(1,1,"f",TokenType::If),
            b'n' => return self.check_keyword(1,2,"il",TokenType::Nil),
            b'o' => return self.check_keyword(1,1,"r",TokenType::Or),
            b'p' => return self.check_keyword(1,4,"rint",TokenType::Print),
            b'r' => return self.check_keyword(1,5,"eturn",TokenType::Return),
            b's' => return self.check_keyword(1,4,"uper",TokenType::Super),
            b't' => {
                if self.current - self.start > 1 {
                    match self.src.as_bytes()[self.start + 1] {
                        b'h' => return self.check_keyword(2, 2, "is", TokenType::This),
                        b'r' => return self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => { return TokenType::Identifier }
                    }
                }
                return TokenType::Identifier;
            }
            b'v' => return self.check_keyword(1,2,"ar",TokenType::Var),
            b'w' => return self.check_keyword(1,4,"hile",TokenType::While),
            _ => { return TokenType::Identifier }
        }
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(&self.peek()) || self.is_digit(&self.peek()) {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() { return self.make_token(TokenType::EndFile); }

        let c = self.advance();
        if self.is_alpha(&c) { return self.identifier(); }
        if self.is_digit(&c) { return self.number(); }

        match c {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),

            b'!' => {
                if self.match_byte(b'=') {
                    return self.make_token(TokenType::BangEqual);
                }
                return self.make_token(TokenType::Bang);
            }
            b'=' => {
                if self.match_byte(b'=') {
                    return self.make_token(TokenType::EqualEqual);
                }
                return self.make_token(TokenType::Equal);
            }
            b'>' => {
                if self.match_byte(b'=') {
                    return self.make_token(TokenType::GreaterEqual);
                }
                return self.make_token(TokenType::Greater);
            }
            b'<' => {
                if self.match_byte(b'=') {
                    return self.make_token(TokenType::LessEqual);
                }
                return self.make_token(TokenType::Less);
            }

            // literals
            b'"' => self.string(),

            _ => Token::error("Unexpected character.", self.line) // &'static str outlives 'src
        }
    }
}