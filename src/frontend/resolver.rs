use crate::backend::treewalk::TWInterp;
use crate::types::ast::*;
use crate::types::token::*;
use crate::util::log::*;

use std::{collections::HashMap};

#[derive(Clone, PartialEq)]
enum FunctionType {
    NONE,
    FUNCTION,
    METHOD,
    INITIALIZER,
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    NONE,
    CLASS,
    SUBCLASS,
}

pub struct Resolver<'t> {
    interpreter: &'t mut TWInterp,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
    had_error: bool
}

impl<'t> Resolver<'t> {
    // ------------- CONSTRUCTOR --------------- 


    pub fn new(interpreter: &'t mut TWInterp) -> Self {
        Self { interpreter, scopes: Vec::default(), current_function: FunctionType::NONE, had_error: false, current_class: ClassType::NONE }
    }

    // -------------   SCOPING   --------------- 

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.len() < 1 { return; }

        let Some(scope) = self.scopes.last_mut() else { return; };
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.len() < 1 { return; }

        let Some(scope) = self.scopes.last_mut() else { return; };
        scope.insert(name.lexeme.clone(), true);
    }

    // -------------  RESOLVING  --------------- 

    fn resolve_function(&mut self, params: &[Token], func: &[Stmt], t: FunctionType) {
        let enclosing_function = self.current_function.clone();
        self.current_function = t;
        
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        
        self.resolve(func);
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve(statements);
                self.end_scope();
            },
            Stmt::Var { name, initializer } => {
                self.declare(name);
                match initializer {
                    Some(i) => {
                        self.resolve_expr(i);
                    },
                    None => {}
                }
                self.define(name);
            },
            Stmt::Function { name, params, body } => {
                self.declare(name);
                self.define(name);
                self.resolve_function(params, &body.borrow(), FunctionType::FUNCTION);
            },
            Stmt::Expression(e) => {
                self.resolve_expr(e);
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch { self.resolve_stmt(else_branch); }
            },
            Stmt::Print(p) => {
                self.resolve_expr(p);
            },
            Stmt::Return { value, keyword } => {
                if self.current_function == FunctionType::NONE {
                    self.had_error = true;
                    alv_error!("Resolver Error: Can't return from top level code on line: {}", keyword.line + 1);
                }

                if let Some(value) = value {
                    if self.current_function == FunctionType::INITIALIZER {
                        self.had_error = true;
                        alv_error!("Resolver Error: Can't return from an initializer on line: {}", keyword.line + 1);
                    }
                
                    self.resolve_expr(value)
                };
            },
            Stmt::While { condition, body } => {
                self.resolve_stmt(body);
                self.resolve_expr(condition);
            },
            Stmt::Class { name, methods, superclass } => {
                let enclosing_class: ClassType = self.current_class;
                self.current_class = ClassType::CLASS;

                self.declare(name);
                self.define(name);

                // check for cycles
                if let Some(Expr::Variable { name: sc_name, .. }) = superclass
                    && sc_name.lexeme == name.lexeme
                {
                    self.had_error = true;
                    alv_error!("Resolver Error: A class can't inherit from itself on line: {}", name.line + 1);
                }

                if let Some(c) = superclass {
                    self.current_class = ClassType::SUBCLASS;
                    self.resolve_expr(c);
                }

                if superclass.is_some() {
                    self.begin_scope();
                    let Some(scope) = self.scopes.last_mut() else {return;}; // TODO: is returning a bug?
                    scope.insert("super".to_string(), true);
                }

                self.begin_scope();
                let Some(scope) = self.scopes.last_mut() else {return;}; // TODO: is returning a bug?
                scope.insert("this".to_string(), true);

                for method in methods {
                    if let Stmt::Function { params, body, name: mname } = method {
                        let ty = if mname.lexeme == "init" { FunctionType::INITIALIZER } else { FunctionType::METHOD };
                        self.resolve_function(params, &body.borrow(), ty);
                    }
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
            }
        }
    }

    fn resolve_local(&mut self, id: usize, name: &Token) {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(id, distance);
                return;
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name, id } => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(&name.lexeme) == Some(&false) {
                        alv_error!("[line {}] Can't read local variable in its own initializer.", name.line + 1);
                        self.had_error = true;
                    }
                }

                self.resolve_local(*id, name);
            },
            Expr::Assign { name, value, id } => {
                self.resolve_expr(value.as_ref());
                self.resolve_local(*id, name)
            },
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },
            Expr::Call { callee, args, .. } => {
                self.resolve_expr(callee);
                for arg in args { self.resolve_expr(arg); }
            },
            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            },
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },
            Expr::Unary { right, .. } => {
                self.resolve_expr(right);
            }
            Expr::Literal { .. } => {},
            Expr::Get { object, .. } => {
                self.resolve_expr(object);
            },
            Expr::Set { object, value, .. } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
            },
            Expr::This { id, keyword } => {
                if self.current_class != ClassType::CLASS {
                    alv_error!("[line {}] Can't use 'this' outside of a class.", keyword.line + 1);
                    self.had_error = true;
                }

                self.resolve_local(*id, keyword);
            },
            Expr::Super { keyword, id, .. } => {
                match self.current_class {
                    ClassType::SUBCLASS => {
                        self.resolve_local(*id, keyword);
                    },
                    ClassType::NONE => {
                        alv_error!("[line {}] Can't use 'super' outside of a class.", keyword.line + 1);
                        self.had_error = true;
                    },
                    ClassType::CLASS => {
                        alv_error!("[line {}] Can't use 'super' in a class with no superclass.", keyword.line + 1);
                        self.had_error = true;
                    }
                }
            }
        }
    }

    pub fn resolve(&mut self, stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
        self.had_error
    }
}