use crate::lexer::*;

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

pub struct Parser {

}
