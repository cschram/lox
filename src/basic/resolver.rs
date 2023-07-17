use super::{
    ast::*,
    error::*,
};
use std::collections::HashMap;

pub type Locals = HashMap<u128, u128>;

pub struct Resolver {
    stack: Vec<HashMap<String, bool>>,
    locals: Locals,
}


impl Resolver {
    pub fn bind(statements: &[Stmt]) -> LoxResult<Locals> {
        let mut resolver = Resolver {
            stack: vec![],
            locals: HashMap::new(),
        };
        for stmt in statements.iter() {
            resolver.bind_stmt(stmt)?;
        }
        Ok(resolver.locals)
    }

    fn bind_stmt(&mut self, stmt: &Stmt) -> LoxResult {
        match stmt {
            Stmt::Block(statements) => {
                self.push();
                for stmt in statements.iter() {
                    self.bind_stmt(stmt)?;
                }
                self.pop();
            },
            Stmt::Var { name, initializer } => {
                self.declare(name.lexeme_str());
                if let Some(init) = initializer {
                    self.bind_expr(init)?;
                }
                self.define(name.lexeme_str());
            },
            Stmt::Fun { name, params, body } => {
                self.define(name.lexeme_str());
                self.push();
                for param in params.iter() {
                    self.define(param.lexeme_str());
                }
                for stmt in body.iter() {
                    self.bind_stmt(stmt)?;
                }
                self.pop();
            },
            Stmt::Expr(expr) => {
                self.bind_expr(expr)?;
            },
            Stmt::IfElse { condition: _, body, else_branch } => {
                self.bind_stmt(body)?;
                if let Some(body) = else_branch {
                    self.bind_stmt(body)?;
                }
            },
            Stmt::Print(expr) => {
                self.bind_expr(expr)?;
            },
            Stmt::Return(expr) => {
                self.bind_expr(expr)?;
            },
            Stmt::WhileLoop { condition, body } => {
                self.bind_expr(condition)?;
                self.bind_stmt(body)?;
            },
        }
        Ok(())
    }

    fn bind_expr(&mut self, expr: &Expr) -> LoxResult {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                if !self.stack.is_empty() && !self.is_defined(&name.lexeme_str()) {
                    return Err(LoxError::Resolution("Attempted to local variable in its own initializer".into()))
                }
                self.resolve_local(expr, name.lexeme_str());
            },
            ExprKind::Assignment { name, value } => {
                self.bind_expr(value)?;
                self.resolve_local(expr, name.lexeme_str());
            },
            ExprKind::Binary { operator: _, left, right } => {
                self.bind_expr(left)?;
                self.bind_expr(right)?;
            },
            ExprKind::Call { callee, arguments } => {
                self.bind_expr(callee)?;
                for arg in arguments.iter() {
                    self.bind_expr(arg)?;
                }
            },
            ExprKind::Grouping(expr) => {
                self.bind_expr(expr)?;
            },
            ExprKind::Logical { operator: _, left, right } => {
                self.bind_expr(left)?;
                self.bind_expr(right)?;
            },
            ExprKind::Unary { operator: _, right } => {
                self.bind_expr(right)?;
            },
            _ => {},
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: String) {
        for (i, frame) in self.stack.iter().rev().enumerate() {
            if frame.contains_key(&name) {
                self.resolve(expr, i as u128);
                break;
            }
        }
    }

    fn resolve(&mut self, expr: &Expr, depth: u128) {
        self.locals.insert(expr.id(), depth);
    }

    fn push(&mut self) {
        self.stack.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn declare(&mut self, name: String) {
        if !self.stack.is_empty() {
            self.peek_mut().insert(name, false);
        }
    }

    fn define(&mut self, name: String) {
        if !self.stack.is_empty() {
            self.peek_mut().insert(name, true);
        }
    }

    fn peek(&self) -> &HashMap<String, bool> {
        let last = self.stack.len() - 1;
        &self.stack[last]
    }

    fn peek_mut(&mut self) -> &mut HashMap<String, bool> {
        let last = self.stack.len() - 1;
        &mut self.stack[last]
    }

    fn is_defined(&self, name: &str) -> bool {
        self.peek()
            .get(name)
            .map(|v| *v)
            .unwrap_or(false)
    }
}
