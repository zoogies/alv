use super::token::{Token, Literal};

use crate::util::log::*;
use crate::types::token::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
        id: usize // used by resolver
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
    Get {
        object: Box<Expr>,
        name: Token
    },
    Set {
        object: Box<Expr>,
        value: Box<Expr>,
        name: Token
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
        name: Token,
        id: usize // used by resolver
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Return{
        keyword: Token,
        value: Option<Expr>
    },
    Var{
        name: Token,
        initializer: Option<Box<Expr>>
    },
    Block{
        statements: Vec<Stmt>
    },
    If{
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>
    },
    While{
        condition: Expr,
        body: Box<Stmt>
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>
    },
    Class {
        name: Token,
        methods: Vec<Stmt> // Stmt::Function
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse_error(token: &Token, msg: String) -> ParseError {
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