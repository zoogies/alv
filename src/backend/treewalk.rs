mod environment;
mod natives;
mod values;

use crate::types::token::*;
use crate::types::ast::*;

use crate::util::log::*;

use self::values::*;
use self::environment::*;
use self::natives::register_natives;

use std::{cell::RefCell, collections::HashMap, process::ExitCode, rc::Rc};

#[derive(Default)]
pub struct TWInterp {
    environment: Rc<RefCell<Environment>>,
    globals: Rc<RefCell<Environment>>,
    locals: Vec<Option<usize>>
}

impl TWInterp {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value } => self.eval_literal(value),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => self.eval_unary(operator, right),
            Expr::Binary { left, operator, right } => self.eval_binary(left, operator, right),
            Expr::Variable { name, id } => self.lookup_variable(name, *id),
            Expr::Assign { name, value, id } => self.eval_assign(name, value, *id),
            Expr::Logical { left, operator, right } => self.eval_logical(left, operator, right),
            Expr::Call { callee, paren, args } => self.eval_call(callee, paren, args),
            Expr::Get { object, name } => self.eval_get(object, name),
            Expr::Set { object, value, name } => self.eval_set(object, value, name),
            Expr::Super { method, id, .. } => self.eval_super(id, method),
            Expr::This { keyword, id } => self.lookup_variable(keyword, *id),
        }
    }

    fn eval_super(&self, id: &usize, method: &Token) -> Result<Value, RuntimeError> {
        // TODO: replace unreachable with runtime errors? I'm tired...
        let Some(dist) = self.locals[*id] else { unreachable!() };
        let Some(Value::Class(superclass)) = self.environment.borrow().get_at(dist, "super") else { unreachable!() };
        let Some(Value::Instance(object)) = self.environment.borrow().get_at(dist - 1, "this") else { unreachable!() };

        let Some(method_fn) = superclass.find_method(&method.lexeme) else {
            return Err(RuntimeError {
                message: format!("Undefined property '{}'.", method.lexeme),
                line: method.line
            });
        };

        Ok(Value::Function(method_fn.bind(&object, method.line)?))
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
            (Value::Instance(x), Value::Instance(y)) => x.equals(y),
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

    fn eval_assign(&mut self, name: &Token, value: &Expr, id: usize) -> Result<Value, RuntimeError> {
        let value = self.evaluate(value)?;
        
        if let Some(dist) = self.locals[id] {
            self.environment.borrow_mut().assign_at(dist, &name.lexeme, value.clone());
        } else {
            self.globals.borrow_mut().assign(&name.lexeme, value.clone());
        };

        Ok(value)
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

    fn eval_get(&mut self, object: &Box<Expr>, name: &Token) -> Result<Value, RuntimeError> {
        match self.evaluate(&object)? {
            Value::Instance(i) => {
                i.get(&name)
            }
            _ => { Err(RuntimeError{message: "Only Instances have properties.".to_string(), line: name.line}) }
        }
    }

    fn eval_set(&mut self, object: &Box<Expr>, value: &Box<Expr>, name: &Token) -> Result<Value, RuntimeError> {
        match self.evaluate(&object)? {
            Value::Instance(i) => {
                let value = self.evaluate(value)?;
                i.set(&name, &value);
                Ok(value)
            }
            _ => { Err(RuntimeError{message: "Only Instances have fields.".to_string(), line: name.line}) }
        }        
    }

    fn eval_call(&mut self, callee: &Box<Expr>, paren: &Token, arguments: &Vec<Expr> ) -> Result<Value, RuntimeError> {
        let callee = self.evaluate(&callee)?;

        let mut args = Vec::new();
        for argument in arguments {
            args.push(self.evaluate(&argument)?);
        }

        let arity = match &callee {
            Value::Function(Function::LoxFunction { decl, .. }) => { decl.params.len() },
            Value::Function(Function::NativeFunction { arity, .. }) => { *arity },
            Value::Class(c) => { if let Some(Function::LoxFunction { decl, .. }) = c.find_method("init") { decl.params.len() } else { 0 } }
            _ => { return Err(RuntimeError{message: "Can only call functions and classes.".to_string(), line:paren.line}); }
        };

        if args.len() != arity {
            return Err(RuntimeError{message: format!("Expected {} arguments but got {}.", arity, args.len()), line:paren.line});
        }

        match callee {
            Value::Function(f) => Ok(self.call_function(&f, &args)?),
            Value::Class(c) => {
                let v = Instance::new(c);

                if let Some(initializer) = v.parent.find_method("init") {
                    if let Ok(r) = initializer.bind(&v, paren.line) {                        
                        self.call_function(&r, &args)?;
                    };
                };

                return Ok(Value::Instance(v))
            },
            _ => unreachable!()
        }
    }

    fn call_function(&mut self, f: &Function, args: &Vec<Value>) -> Result<Value, RuntimeError> {
        match f {
            Function::NativeFunction { imp, .. } => {
                Ok(imp(self, args)?)
            },
            Function::LoxFunction { decl, closure, is_initializer, .. } => {
                let e = Rc::new(RefCell::new(Environment::from_enclosing(closure)));

                for (i, arg) in args.iter().enumerate() {
                    e.borrow_mut().define(decl.params.get(i).expect("Params didn't match args").lexeme.clone(), arg.clone());
                }

                let r = self.execute_block(&decl.body, e);
                if let Err(Interrupt::Error(err)) = r {
                    return Err(err);
                }

                if *is_initializer && let Some(v) = closure.borrow().get_at(0, "this") {
                    return Ok(v);
                }

                match r {
                    Ok(..) => {
                        Ok(Value::Nil)
                    },
                    Err(Interrupt::Return { value, .. }) => {
                        match value {
                            None => {
                                Ok(Value::Nil)
                            },
                            Some(s) => {
                                Ok(s)
                            }
                        }
                    },

                    // unreachable.
                    Err(Interrupt::Error(e)) => {
                        Err(e)
                    }
                }
            }
        }
    }

    fn execute_block(&mut self, blocks: &Vec<Stmt>, env: Rc<RefCell<Environment>>) -> Result<(), Interrupt> {
        let prev = Rc::clone(&self.environment);
        self.environment = env;
        let res = blocks.iter().try_for_each(|s| self.execute(s));
        self.environment = prev;

        res
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Interrupt> {
        match stmt {
            Stmt::Print(pstmt) => {
                let v: Value = self.evaluate(&*pstmt)?;
                alv_log!("{}",alv_stringify(&v));
                Ok(())
            },
            Stmt::Expression(estmt) => {
                self.evaluate(&*estmt)?;
                Ok(())
            },
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil
                };
                self.environment.borrow_mut().define(name.lexeme.to_string(), value);
                Ok(())
            },
            Stmt::Block { statements } => {
                let e = Rc::new(RefCell::new(Environment::from_enclosing(&Rc::clone(&self.environment))));

                Ok(self.execute_block(statements, e)?)
            },
            Stmt::If { condition, then_branch, else_branch } => {
                let condition_value = self.evaluate(condition)?;
                if self.is_truthy(&condition_value) {
                    self.execute(&then_branch)?;
                }
                else if else_branch.is_some() {
                    self.execute(&else_branch.as_ref().unwrap())?;
                }

                Ok(())
            },
            Stmt::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate(condition)?;

                    if self.is_truthy(&condition_value) { self.execute(body)?; }
                    else { break; }
                }
                Ok(())
            },
            Stmt::Function (decl) => {
                self.environment.borrow_mut().define(
                    decl.name.lexeme.clone(),
                    Value::Function(
                        Function::LoxFunction { 
                            decl: Rc::clone(decl),
                            closure: Rc::clone(&self.environment),
                            is_initializer: false
                        }
                    )
                );

                Ok(())
            },
            Stmt::Return { keyword, value } => {
                match value {
                    None => {
                        return Err(Interrupt::Return { keyword: keyword.clone(), value: None });
                    },
                    Some(v) => {
                        Err(Interrupt::Return{keyword: keyword.clone(), value: Some(self.evaluate(&v)?)})
                    }
                }
            },
            Stmt::Class { name, methods, superclass } => {
                let superclass = match superclass {
                    Some(expr) => match self.evaluate(expr)? {
                        Value::Class(c) => Some(c),
                        _ => return Err(Interrupt::Error(RuntimeError { message: "Superclass must be a class.".to_string(), line: name.line }))
                    },
                    None => None,
                };
                
                self.environment.borrow_mut().define(name.lexeme.clone(), Value::Nil);

                // TODO: OPTIMIZATION: don't do this unless superclass.is_some()
                let prev = Rc::clone(&self.environment);

                if let Some(sc) = &superclass {
                    self.environment = Rc::new(RefCell::new(Environment::from_enclosing(&Rc::clone(&self.environment))));

                    self.environment.borrow_mut().define("super".to_string(), Value::Class(Rc::clone(sc)));
                };

                let mut meths: HashMap<String, Function> = HashMap::new();
                for method in methods {
                    if let Stmt::Function (decl) = method {
                        meths.insert(
                            decl.name.lexeme.clone(),
                            Function::LoxFunction {
                                decl: Rc::clone(decl),
                                closure: Rc::clone(&self.environment),
                                is_initializer: decl.name.lexeme.eq("init")
                            }
                        );
                    }
                }

                let klass = Value::Class(Rc::new(Class::new(&name.lexeme, meths, superclass)));

                self.environment = prev;

                self.environment.borrow_mut().assign(&name.lexeme, klass);
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
                    match error {
                        Interrupt::Error(error) => {
                            alv_error!("Runtime error on line {}! {}", error.line + 1, error.message);
                        },
                        Interrupt::Return{keyword, ..} => {
                            alv_error!("Runtime error on line {}! Trying to return a value from non-returning scoped block.", keyword.line + 1);
                        }
                    }
                    return ExitCode::FAILURE;
                }
            }
        }
        ExitCode::SUCCESS
    }

    pub fn new(prealloc: &usize) -> Self {
        let mut s = Self {
            environment: Rc::new(RefCell::new(Environment::default())),
            globals: Rc::new(RefCell::new(Environment::default())),
            locals: vec![None; *prealloc]
        };
        s.environment = Rc::clone(&s.globals);

        register_natives(&mut s.globals.borrow_mut());

        s
    }

    // RESOLVER

    pub fn resolve(&mut self, id: usize, depth: usize) {
        if id >= self.locals.len() {
            self.locals.resize(id + 1, None);
        }

        self.locals[id] = Some(depth);
    }

    fn lookup_variable(&self, name: &Token, id: usize) -> Result<Value, RuntimeError> {
        let found = if let Some(dist) = self.locals[id] {
            self.environment.borrow().get_at(dist, &name.lexeme)
        } else {
            self.globals.borrow().get(&name.lexeme)
        };

        if let Some(found) = found {
            return Ok(found);
        } else {
            return Err(RuntimeError{message: format!("lookup_variable failed for variable: '{}'", name.lexeme), line: name.line})
            // TODO: not the most accurate error message? I'm spitballing here
        }
    }

}