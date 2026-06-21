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
    pub fn from_ident(text: &str) -> Self {
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
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<Literal>,
}

impl<'p> Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, lit: Option<Literal>) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            literal: lit,
        }
    }
}