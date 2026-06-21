use crate::types::token::Token;
use crate::types::ast::Stmt;
use super::TWInterp;

#[derive(Debug, Clone)]
pub enum Function {
    NativeFunction{arity: usize, imp: fn(&mut TWInterp, &[Value]) -> Result<Value, RuntimeError>},
    LoxFunction{name: Token, params: Vec<Token>, body: Vec<Stmt>}
}

// currently shadows literal, but with global strings
#[derive(Debug, Clone)]
pub enum Value {
    String(String), // heap strings for runtime?
    Number(f64),
    Boolean(bool),
    Nil,
    Function(Function)
}

// needs new lifetime specifier later if you add AST/token slices
pub struct RuntimeError {
    pub message: String,
    pub line: usize
}

pub fn alv_stringify(value: &Value) -> String {
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
            },
            Value::Function(f) => {
                match f {
                    Function::NativeFunction { .. } => { "<NATIVE FUNCTION>".to_string() },
                    Function::LoxFunction { name, .. } => { format!("<Fn {}>", name.lexeme) }
                }
            }
        }
    }