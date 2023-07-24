use super::{ast::*, error::*};
use std::collections::HashMap;

pub type Locals = HashMap<usize, usize>;

pub struct Resolver {
    locals_stack: Vec<HashMap<String, bool>>,
    locals: Locals,
    functions_stack: Vec<()>,
}

impl Resolver {
    pub fn bind(statements: &[Stmt]) -> LoxResult<Locals> {
        let mut resolver = Resolver {
            locals_stack: vec![],
            locals: HashMap::new(),
            functions_stack: vec![],
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
            }
            Stmt::Var { name, initializer } => {
                if self.has_name(&name.lexeme_str()) {
                    return Err(LoxError::Runtime(format!("Cannot redeclare variable \"{}\" in the same scope", name.lexeme_str())))
                }
                self.declare(name.lexeme_str());
                if let Some(init) = initializer {
                    self.bind_expr(init)?;
                }
                self.define(name.lexeme_str());
            }
            Stmt::Fun { name, params, body } => {
                self.define(name.lexeme_str());
                self.functions_stack.push(());
                self.push();
                for param in params.iter() {
                    self.define(param.lexeme_str());
                }
                for stmt in body.iter() {
                    self.bind_stmt(stmt)?;
                }
                self.pop();
                self.functions_stack.pop();
            }
            Stmt::Expr(expr) => {
                self.bind_expr(expr)?;
            }
            Stmt::IfElse {
                condition: _,
                body,
                else_branch,
            } => {
                self.bind_stmt(body)?;
                if let Some(body) = else_branch {
                    self.bind_stmt(body)?;
                }
            }
            Stmt::Print(expr) => {
                self.bind_expr(expr)?;
            }
            Stmt::Return(expr) => {
                if self.functions_stack.is_empty() {
                    return Err(LoxError::Runtime("Cannot return from global scope".into()));
                }
                self.bind_expr(expr)?;
            }
            Stmt::WhileLoop { condition, body } => {
                self.push();
                self.bind_expr(condition)?;
                self.bind_stmt(body)?;
                self.pop();
            }
            Stmt::Class { name, methods } => {
                self.declare(name.lexeme_str());
                self.define(name.lexeme_str());
            }
        }
        Ok(())
    }

    fn bind_expr(&mut self, expr: &Expr) -> LoxResult {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                if !self.locals_stack.is_empty() && !self.is_initialized(&name.lexeme_str()) {
                    return Err(LoxError::Resolution(
                        "Attempted to resolve variable in its own initializer".into(),
                    ));
                }
                self.resolve_local(expr, name.lexeme_str());
            }
            ExprKind::Assignment { name, value } => {
                self.bind_expr(value)?;
                self.resolve_local(expr, name.lexeme_str());
            }
            ExprKind::Binary {
                operator: _,
                left,
                right,
            } => {
                self.bind_expr(left)?;
                self.bind_expr(right)?;
            }
            ExprKind::Call { callee, arguments } => {
                self.bind_expr(callee)?;
                for arg in arguments.iter() {
                    self.bind_expr(arg)?;
                }
            }
            ExprKind::Grouping(expr) => {
                self.bind_expr(expr)?;
            }
            ExprKind::Logical {
                operator: _,
                left,
                right,
            } => {
                self.bind_expr(left)?;
                self.bind_expr(right)?;
            }
            ExprKind::Unary { operator: _, right } => {
                self.bind_expr(right)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: String) {
        for (i, frame) in self.locals_stack.iter().rev().enumerate() {
            if frame.contains_key(&name) {
                self.resolve(expr, i);
                break;
            }
        }
    }

    fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.id(), depth);
    }

    fn push(&mut self) {
        self.locals_stack.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.locals_stack.pop();
    }

    fn declare(&mut self, name: String) {
        if !self.locals_stack.is_empty() {
            self.peek_mut().insert(name, false);
        }
    }

    fn define(&mut self, name: String) {
        if !self.locals_stack.is_empty() {
            self.peek_mut().insert(name, true);
        }
    }

    fn peek(&self) -> &HashMap<String, bool> {
        let last = self.locals_stack.len() - 1;
        &self.locals_stack[last]
    }

    fn peek_mut(&mut self) -> &mut HashMap<String, bool> {
        let last = self.locals_stack.len() - 1;
        &mut self.locals_stack[last]
    }

    fn has_name(&self, name: &str) -> bool {
        if self.locals_stack.is_empty() {
            false
        } else {
            self.peek().contains_key(name)
        }
    }

    fn is_initialized(&self, name: &str) -> bool {
        self.peek().get(name).copied().unwrap_or(true)
    }
}

#[cfg(test)]
mod test {
    use super::super::super::test_scripts::*;
    use super::super::parser::*;
    use super::*;

    fn local_keys(locals: &Locals) -> Vec<&usize> {
        let mut keys = locals.keys().collect::<Vec<&usize>>();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn block_scope() -> LoxResult {
        let ParseResult {
            statements,
            errors: _,
        } = parse(BLOCK_SCOPE_TEST);
        let locals = Resolver::bind(&statements)?;
        let keys = local_keys(&locals);
        assert_eq!(locals.len(), 1);
        assert_eq!(locals.get(keys[0]), Some(&0));
        Ok(())
    }

    #[test]
    fn for_loop() -> LoxResult {
        let ParseResult {
            statements,
            errors: _,
        } = parse(FOR_LOOP_TEST);
        let locals = Resolver::bind(&statements)?;
        let keys = local_keys(&locals);
        assert_eq!(locals.len(), 4);
        assert_eq!(locals.get(keys[0]), Some(&1));
        assert_eq!(locals.get(keys[1]), Some(&2));
        assert_eq!(locals.get(keys[2]), Some(&2));
        assert_eq!(locals.get(keys[3]), Some(&3));
        Ok(())
    }

    #[test]
    fn function() -> LoxResult {
        let ParseResult {
            statements,
            errors: _,
        } = parse(FUNCTION_TEST);
        let locals = Resolver::bind(&statements)?;
        let keys = local_keys(&locals);
        assert_eq!(locals.len(), 2);
        assert_eq!(locals.get(keys[0]), Some(&1));
        assert_eq!(locals.get(keys[1]), Some(&0));
        Ok(())
    }

    #[test]
    fn function_closure() -> LoxResult {
        let ParseResult {
            statements,
            errors: _,
        } = parse(FUNCTION_CLOSURE_TEST);
        let locals = Resolver::bind(&statements)?;
        let keys = local_keys(&locals);
        assert_eq!(locals.len(), 4);
        assert_eq!(locals.get(keys[0]), Some(&1));
        assert_eq!(locals.get(keys[1]), Some(&1));
        assert_eq!(locals.get(keys[2]), Some(&1));
        assert_eq!(locals.get(keys[3]), Some(&0));
        Ok(())
    }

    #[test]
    fn shadowing() -> LoxResult {
        let ParseResult {
            statements,
            errors: _,
        } = parse(SHADOWING_TEST);
        let locals = Resolver::bind(&statements)?;
        let keys = local_keys(&locals);
        assert_eq!(locals.len(), 2);
        assert_eq!(locals.get(keys[0]), Some(&0));
        assert_eq!(locals.get(keys[1]), Some(&0));
        Ok(())
    }
}
