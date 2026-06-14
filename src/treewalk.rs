use crate::{lexer::*, log::{alv_error, alv_log}, parser::*};

use std::{collections::HashMap, process::ExitCode};

// currently shadows literal, but with global strings
#[derive(Debug, Clone)]
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

#[derive(Default)]
pub struct TWInterp {
    environment: HashMap<String,Value>
}

impl TWInterp {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value } => self.eval_literal(value),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => self.eval_unary(operator, right),
            Expr::Binary { left, operator, right } => self.eval_binary(left, operator, right),
            Expr::Variable { name } => self.eval_variable(name),
            Expr::Assign { name, value } => self.eval_assign(name, value),
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

    fn eval_unary(&mut self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
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

    fn eval_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
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

    fn eval_variable(&self, name: &Token) -> Result<Value, RuntimeError> {
        match self.environment.get(name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError { message: "Undefined variable.", line: name.line })
        }
    }

    fn eval_assign(&mut self, name: &Token, value: &Expr) -> Result<Value, RuntimeError> {
        let value = self.evaluate(value)?;
        
        if self.environment.contains_key(name.lexeme) {
            self.environment.entry(name.lexeme.to_string()).and_modify(|v| *v = value.clone());
        }
        else {
            return Err(RuntimeError {message: "Undefined variable TODO INSERT LEXEME NAME", line: name.line});
        }

        Ok(value)
    }

    fn stringify(&self, value: &Value) -> String {
        match value {
            Value::String(s) => {
                s.clone()
            },
            Value::Boolean(b) => {
                b.to_string()
            },
            Value::Number(n) => {
                n.to_string()
            }
            Value::Nil => {
                "Nil".to_string()
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::PrintStmt(pstmt) => {
                let v: Value = self.evaluate(&*pstmt)?;
                alv_log!("{}",self.stringify(&v));
                Ok(())
            },
            Stmt::ExpressionStmt(estmt) => {
                self.evaluate(&*estmt)?;
                Ok(())
            },
            Stmt::VarStmt { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil
                };
                self.environment.insert(name.lexeme.to_string(), value);
                Ok(())
            }
        }
    }

    // TODO: bubble out an exit code?
    pub fn interpret(&mut self, stmts: &Vec<Stmt>) -> ExitCode {
        for stmt in stmts {
            match self.execute(stmt) {
                Ok(_v) => {
                    // alv_log!("Treewalk output: {:?}", v); // TODO: FIX
                },
                Err(error) => {
                    alv_error!("Runtime error on line {}! {}", error.line + 1, error.message);
                    return ExitCode::FAILURE;
                }
            }
        }
        ExitCode::SUCCESS
    }

}
