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
