use crate::{lexer::*, log::{alv_error, alv_log}, parser::*};
use std::time::{SystemTime, UNIX_EPOCH};

use std::{cell::RefCell, collections::HashMap, default, process::ExitCode, rc::Rc};

#[derive(Debug, Clone)]
enum Function {
    NativeFunction(fn(&mut TWInterp, &[Value]) -> Result<Value, RuntimeError>)
}

// currently shadows literal, but with global strings
#[derive(Debug, Clone)]
pub enum Value {
    String(String), // heap strings for runtime?
    Number(f64),
    Boolean(bool),
    Nil,
    Callable { arity: usize, imp: Function },
}

// needs new lifetime specifier later if you add AST/token slices
pub struct RuntimeError {
    pub message: String,
    pub line: usize
}

#[derive(Default)]
pub struct Environment {
    environment: HashMap<String,Value>,
    enclosing: Option<Rc<RefCell<Environment>>>
}

impl Environment {
    pub fn get(&self, k: &str) -> Option<Value> {
        match self.environment.get(k) {
            Some(v) => Some(v.clone()),
            None => match &self.enclosing {
                Some(parent) => parent.borrow().get(k),
                None => None
            }
        }
    }

    pub fn define(&mut self, k: String, v: Value) {
        self.environment.insert(k, v);
    }

    pub fn assign(&mut self, k: &str, v: Value) -> bool {
        if self.environment.contains_key(k) {
            self.environment.insert(k.to_string(), v);
            true
        }
        else {
            match &self.enclosing {
                Some(parent) => parent.borrow_mut().assign(k, v),
                None => false,
            }
        }
    }
}

#[derive(Default)]
pub struct TWInterp {
    environment: Rc<RefCell<Environment>>,
    globals: Environment
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
            Expr::Logical { left, operator, right } => self.eval_logical(left, operator, right),
            Expr::Call { callee, paren, args } => self.eval_call(callee, paren.clone(), args),
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
                    _ => Err(RuntimeError { message: "Operand must be a number.".to_string(), line: operator.line })
                }
            },
            TokenType::Bang => { return Ok(Value::Boolean(!self.is_truthy(&right))); }
            _ => Err(RuntimeError { message: "Unimplemented or invalid unary expression operator".to_string(), line: operator.line })
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
            _ => Err(RuntimeError {message: "Operands must be numbers".to_string(), line: operator.line } )
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
                _ => Err(RuntimeError {message: "Operands must be numbers or strings".to_string(), line: operator.line } )
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
            _ => { Err(RuntimeError {message: "Unimplemented or invalid binary expression operator.".to_string(), line: operator.line}) }
        }
    }

    fn eval_variable(&mut self, name: &Token) -> Result<Value, RuntimeError> {
        match self.environment.borrow().get(&name.lexeme) {
            Some(v) => Ok(v),
            None => Err(RuntimeError { message: "Undefined variable.".to_string(), line: name.line })
        }
    }

    fn eval_assign(&mut self, name: &Token, value: &Expr) -> Result<Value, RuntimeError> {
        let value = self.evaluate(value)?;
        
        if self.environment.borrow_mut().assign(&name.lexeme, value.clone()) {
            Ok(value)
        }
        else {
            Err(RuntimeError {message: "Undefined variable TODO INSERT LEXEME NAME".to_string(), line: name.line}) // TODO: DO THIS YOU THUG <- don't call me that.
        }
    }

    fn eval_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let left = self.evaluate(left)?;

        if operator.token_type == TokenType::Or {
            if self.is_truthy(&left) { return Ok(left); }
        }
        else {
            if !self.is_truthy(&left) { return Ok(left); }
        }

        self.evaluate(right)
    }

    fn eval_call(&mut self, callee: &Box<Expr>, paren: Token, arguments: &Vec<Expr> ) -> Result<Value, RuntimeError> {
        let callee = self.evaluate(&callee)?;

        let mut args = Vec::new();
        for argument in arguments {
            args.push(self.evaluate(&argument)?);
        }


        let Value::Callable{arity, imp} = &callee else {
            return Err(RuntimeError{message: "Can only call functions and classes.".to_string(), line:paren.line});
        };

        if args.len() != *arity {
            return Err(RuntimeError{message: format!("Expected {} arguments but got {}.", arity, args.len()), line:paren.line});
        }

        Ok(self.call_function(&callee, &args)?)
    }

    fn call_function(&mut self, f: &Value, args: &Vec<Value>) -> Result<Value, RuntimeError> {
        Ok(Value::Nil) // TODO
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
            },
            Value::Callable {arity: _arity, imp: _imp} => {
                "Function".to_string() // TODO
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
                self.environment.borrow_mut().define(name.lexeme.to_string(), value);
                Ok(())
            },
            Stmt::BlockStmt { statements } => {
                self.environment = Rc::new(RefCell::new(Environment {
                    environment: HashMap::new(),
                    enclosing: Some(Rc::clone(&self.environment))
                }));

                for statement in statements {
                    self.execute(statement)?;
                }

                let parent = self.environment.borrow().enclosing.clone().unwrap();
                self.environment = parent;

                Ok(())
            },
            Stmt::IfStmt { condition, then_branch, else_branch } => {
                let condition_value = self.evaluate(condition)?;
                if self.is_truthy(&condition_value) {
                    self.execute(&then_branch)?;
                }
                else if else_branch.is_some() {
                    self.execute(&else_branch.as_ref().unwrap())?;
                }

                Ok(())
            },
            Stmt::WhileStmt { condition, body } => {
                loop {
                    let condition_value = self.evaluate(condition)?;

                    if self.is_truthy(&condition_value) { self.execute(body)?; }
                    else { break; }
                }
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

    pub fn new() -> Self {
        let mut s = Self {
            environment: Rc::new(RefCell::new(Environment::default())),
            globals: Environment::default()
        };

        register_natives(&mut s.globals);

        s
    }

}

// NATIVE FUNCTIONS

fn register_natives(env: &mut Environment) {
    env.define("clock".to_string(), Value::Callable { arity: 0, imp: Function::NativeFunction(clock) });
}

fn clock(_interp: &mut TWInterp, _args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(
        Value::Number(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as f64)
    )
}
