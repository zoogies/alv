enum TokenType {
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
    Return, Super, This, Var, While
}

struct Token<'program> {
    token_type: TokenType,
    lexeme: &'program str,
    line: usize,
}

impl<'program> Token<'program> {
    fn new(token_type: TokenType, lexeme: &'program str, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

struct Lexer<'program> {
    input: &'program str, 
    tokens: Vec<Token<'program>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'program> Lexer<'program> {
    fn new(input: &'program str) -> Self {
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
        self.input.as_bytes()[self.current] as char
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(
            Token::new(
                token_type, 
                &self.input[self.start..self.current], 
                self.line
            )
        );
    }

    fn scan_token(&mut self) {
        let c: char = advance();

        match c {
            '(' => add_token(TokenType::LeftParen),
            ')' => add_token(TokenType::RightParen),
            '{' => add_token(TokenType::LeftBrace),
            '}' => add_token(TokenType::RightBrace),
            '+' => add_token(TokenType::Plus),
            '-' => add_token(TokenType::Minus),
            '*' => add_token(TokenType::Star),
            ';' => add_token(TokenType::Semicolon),
            '.' => add_token(TokenType::Dot),
            ',' => add_token(TokenType::Comma),
        }
    }
}