use crate::{lexer::*, log::{alv_error, alv_log}, parser::*};

use std::process::ExitCode;

// currently shadows literal, but with global strings
#[derive(Debug)]
pub enum Value {
    String(String), // heap strings for runtime?
    Number(f64),
    Boolean(bool),
    Nil
}

// needs new lifetime specifier later if you add AST/token slices
pub struct RuntimeError {
    pub message: &'static str,
    pub line: usize
}

pub struct TWInterp;

impl TWInterp {
    pub fn evaluate(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value } => self.eval_literal(value),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => self.eval_unary(operator, right),
            Expr::Binary { left, operator, right } => self.eval_binary(left, operator, right),
        }
    }

    fn eval_literal(&self, value: &Literal) -> Result<Value, RuntimeError> {
        match value {
            Literal::String(s) => { return Ok(Value::String(s.to_string())) },
            Literal::Number(n)  => { return Ok(Value::Number(*n)) },
            Literal::Bool(b)   => { return Ok(Value::Boolean(*b)) },
            Literal::Nil => { return Ok(Value::Nil) }
        }
    }

    fn is_truthy(&self, v: &Value) -> bool {
        match v {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true
        }
    }

    fn eval_unary(&self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let right = self.evaluate(right)?;

        match operator.token_type {

            TokenType::Minus => {
                match right {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(RuntimeError { message: "Operand must be a number.", line: operator.line })
                }
            },
            TokenType::Bang => { return Ok(Value::Boolean(!self.is_truthy(&right))); }
            _ => Err(RuntimeError { message: "Unimplemented or invalid unary expression operator", line: operator.line })
        }
    }

    fn is_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Nil, Value::Nil) => true,
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            _ => false
        }
    }

    fn number_operands(&self, operator: &Token, left: &Value, right: &Value) -> Result<(f64, f64), RuntimeError> {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok((*l, *r)),
            _ => Err(RuntimeError {message: "Operands must be numbers", line: operator.line } )
        }
    }

    fn eval_binary(&self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Number(l - r))
            },
            TokenType::Slash => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Number(l / r))
            },
            TokenType::Star => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Number(l * r))
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{l}{r}"))),
                _ => Err(RuntimeError {message: "Operands must be numbers or strings", line: operator.line } )
            }
            TokenType::Greater => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Boolean(l > r))
            },
            TokenType::GreaterEqual => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Boolean(l >= r))
            },
            TokenType::Less => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Boolean(l < r))
            },
            TokenType::LessEqual => {
                let (l, r) = self.number_operands(operator, &left, &right)?;
                Ok(Value::Boolean(l <= r))
            },
            TokenType::BangEqual => Ok(Value::Boolean(!self.is_equal(&left, &right))),
            TokenType::EqualEqual => Ok(Value::Boolean(self.is_equal(&left, &right))),
            _ => { Err(RuntimeError {message: "Unimplemented or invalid binary expression operator.", line: operator.line}) }
        }
    }

    // TODO: bubble out an exit code?
    pub fn interpret(&self, expr: &Expr) -> ExitCode {
        let v = self.evaluate(expr);
        match v {
            Ok(v) => {
                alv_log!("Treewalk output: {:?}", v); // TODO: FIX
                return ExitCode::SUCCESS;
            },
            Err(error) => {
                alv_error!("Runtime error on line {}! {}", error.line, error.message);
                return ExitCode::FAILURE;
            }
        }
    }

}
