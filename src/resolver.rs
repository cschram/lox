use crate::{error::*, expr::*, scanner::*, stmt::*};
use std::collections::HashMap;

pub type Locals = HashMap<usize, usize>;

#[derive(PartialEq, Clone, Copy)]
enum FunctionType {
    Function,
    Constructor,
    Method,
}

#[derive(PartialEq, Clone, Copy)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver {
    locals_stack: Vec<HashMap<String, bool>>,
    locals: Locals,
    functions_stack: Vec<FunctionType>,
    current_class: ClassType,
}

impl Resolver {
    pub fn bind(statements: &[Stmt]) -> LoxResult<Locals> {
        let mut resolver = Resolver {
            locals_stack: vec![],
            locals: HashMap::new(),
            functions_stack: vec![],
            current_class: ClassType::None,
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
                    return Err(LoxError::Runtime(format!(
                        "Cannot redeclare variable \"{}\" in the same scope",
                        name.lexeme_str()
                    )));
                }
                self.declare(name.lexeme_str());
                if let Some(init) = initializer {
                    self.bind_expr(init)?;
                }
                self.define(name.lexeme_str());
            }
            Stmt::Fun { name, params, body } => {
                self.resolve_function(name, params, body, FunctionType::Function)?;
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
                if self.functions_stack[self.functions_stack.len() - 1] == FunctionType::Constructor
                {
                    return Err(LoxError::Resolution(
                        "Cannot return from constructor".into(),
                    ));
                }
                self.bind_expr(expr)?;
            }
            Stmt::WhileLoop { condition, body } => {
                self.push();
                self.bind_expr(condition)?;
                self.bind_stmt(body)?;
                self.pop();
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                self.current_class = ClassType::Class;
                self.declare(name.lexeme_str());
                if let Some(superclass) = superclass {
                    if let ExprKind::Identifier(supername) = &superclass.kind {
                        if supername.lexeme_str() == name.lexeme_str() {
                            return Err(LoxError::Resolution(format!(
                                "Class \"{}\" cannot inherit from itself",
                                name.lexeme_str()
                            )));
                        } else {
                            self.bind_expr(superclass)?;
                        }
                    } else {
                        unreachable!("Expected an identifier");
                    }
                }
                for method in methods.iter() {
                    if let Stmt::Fun { name, params, body } = method {
                        self.resolve_function(
                            name,
                            params,
                            body,
                            if name.lexeme_str() == *"init" {
                                FunctionType::Constructor
                            } else {
                                FunctionType::Method
                            },
                        )?;
                    }
                }
                self.define(name.lexeme_str());
                self.current_class = ClassType::None;
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
            ExprKind::This => {
                if self.current_class == ClassType::None {
                    return Err(LoxError::Resolution(
                        "Cannot use \"this\" outside of a class".into(),
                    ));
                }
            }
            ExprKind::Super(..) => {
                if self.current_class == ClassType::None {
                    return Err(LoxError::Resolution(
                        "Cannot use \"super\" outside of a class".into(),
                    ));
                }
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

    fn resolve_function(
        &mut self,
        name: &Token,
        params: &[Token],
        body: &[Stmt],
        func_type: FunctionType,
    ) -> LoxResult {
        self.define(name.lexeme_str());
        self.functions_stack.push(func_type);
        self.push();
        if func_type == FunctionType::Method {
            self.define("this".into());
        }
        for param in params.iter() {
            self.define(param.lexeme_str());
        }
        for stmt in body.iter() {
            self.bind_stmt(stmt)?;
        }
        self.pop();
        self.functions_stack.pop();
        Ok(())
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
    use super::*;
    use crate::{parser::*, test_scripts::*};

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

    #[test]
    fn class() -> LoxResult {
        let ParseResult {
            statements,
            errors: _,
        } = parse(CLASS_TEST);
        let locals = Resolver::bind(&statements)?;
        let keys = local_keys(&locals);
        assert_eq!(locals.len(), 1);
        assert_eq!(locals.get(keys[0]), Some(&0));
        Ok(())
    }

    #[test]
    fn invalid_this() {
        let ParseResult {
            statements,
            errors: _,
        } = parse(
            r#"
            fun invalid_this() {
                return this;
            }
            invalid_this();
        "#,
        );
        let result = Resolver::bind(&statements);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(LoxError::Resolution(message)) if message == "Cannot use \"this\" outside of a class".to_string()
        ));
    }

    #[test]
    fn constructor_return() {
        let ParseResult {
            statements,
            errors: _,
        } = parse(
            r#"
            class InvalidReturn {
                init() {
                    return "foo";
                }
            }
        "#,
        );
        let result = Resolver::bind(&statements);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(LoxError::Resolution(message)) if message == "Cannot return from constructor".to_string()
        ));
    }
}
