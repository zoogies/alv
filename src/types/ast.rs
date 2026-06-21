use super::token::{Token, Literal};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
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
    }
}