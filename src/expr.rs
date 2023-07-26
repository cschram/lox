use super::{
    scanner::{Token, TokenKind},
    state::LoxState,
    environment::{ScopeHandle, LoxVars, GLOBAL_SCOPE},
    error::*,
    value::{LoxValue, LoxObject},
};
use std::{
    cell::RefCell,
    fmt,
    rc::Rc,
};

thread_local! {
    static EXPR_COUNT: RefCell<usize> = const { RefCell::new(0) };
}

fn get_expr_id() -> usize {
    let mut id = 0;
    EXPR_COUNT.with(|cell| {
        id = cell.take();
        cell.replace(id + 1);
    });
    id
}

#[derive(PartialEq, Clone)]
pub enum ExprKind {
    Literal(Token),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Identifier(Token),
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Get {
        left: Box<Expr>,
        right: Token,
    },
    Set {
        object: Box<Expr>,
        identifier: Token,
        value: Box<Expr>,
    },
    This(Token),
}

#[derive(PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    _id: usize,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self {
            kind,
            _id: get_expr_id(),
        }
    }

    pub fn id(&self) -> usize {
        self._id
    }

    pub fn eval(&self, state: &mut LoxState, scope: ScopeHandle) -> LoxResult<LoxValue> {
        match &self.kind {
            ExprKind::Literal(value) => Ok(LoxValue::from(value.clone())),
            ExprKind::Unary { operator, right } => match operator.kind {
                TokenKind::Bang => {
                    // let right_value = self.evaluate_expr(scope, right)?.is_truthy();
                    let right_value = right.eval(state, scope)?.is_truthy();
                    Ok(LoxValue::Boolean(!right_value))
                }
                _ => Err(LoxError::Runtime(format!(
                    "Unknown unary operator \"{}\"",
                    operator
                ))),
            },
            ExprKind::Binary {
                operator,
                left,
                right,
            } => {
                let left_value = left.eval(state, scope)?;
                let right_value = right.eval(state, scope)?;
                match operator.kind {
                    TokenKind::Plus => {
                        if left_value.is_string() || right_value.is_string() {
                            Ok(LoxValue::String(format!(
                                "{}{}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        } else if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Number(
                                left_value.get_number()? + right_value.get_number()?,
                            ))
                        } else {
                            Err(LoxError::Runtime(format!(
                                "Invalid operands {} + {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    }
                    TokenKind::Minus => Ok(LoxValue::Number(
                        left_value.get_number()? - right_value.get_number()?,
                    )),
                    TokenKind::Star => Ok(LoxValue::Number(
                        left_value.get_number()? * right_value.get_number()?,
                    )),
                    TokenKind::Slash => Ok(LoxValue::Number(
                        left_value.get_number()? / right_value.get_number()?,
                    )),
                    TokenKind::Greater => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number()? > right_value.get_number()?,
                            ))
                        } else {
                            Err(LoxError::Runtime(format!(
                                "Invalid operands {} > {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    }
                    TokenKind::GreaterEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number()? >= right_value.get_number()?,
                            ))
                        } else {
                            Err(LoxError::Runtime(format!(
                                "Invalid operands {} >= {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    }
                    TokenKind::Less => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number()? < right_value.get_number()?,
                            ))
                        } else {
                            Err(LoxError::Runtime(format!(
                                "Invalid operands {} < {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    }
                    TokenKind::LessEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number()? <= right_value.get_number()?,
                            ))
                        } else {
                            Err(LoxError::Runtime(format!(
                                "Invalid operands {} <= {}",
                                left_value.to_string(),
                                right_value.to_string(),
                            )))
                        }
                    }
                    TokenKind::EqualEqual => Ok(LoxValue::Boolean(left_value == right_value)),
                    TokenKind::BangEqual => Ok(LoxValue::Boolean(left_value != right_value)),
                    _ => Err(LoxError::Runtime(format!(
                        "Unknown binary operator \"{}\"",
                        operator
                    ))),
                }
            },
            ExprKind::Grouping(inner) => inner.eval(state, scope),
            ExprKind::Identifier(name) => {
                let scope = match state.locals.get(&self._id) {
                    Some(depth) => state.env.ancestor_scope(scope, *depth).unwrap_or_else(|| {
                        panic!("Invalid ancestor scope for \"{}\"", name.lexeme_str())
                    }),
                    None => GLOBAL_SCOPE,
                };
                state.env
                    .get(Some(scope), &name.lexeme_str())
                    .ok_or(LoxError::Runtime(format!(
                        "Undefined variable \"{}\"",
                        name.lexeme_str()
                    )))
            }
            ExprKind::Assignment { name, value } => {
                let val = value.eval(state, scope)?;
                let scope = match state.locals.get(&self._id) {
                    Some(distance) => {
                        state.env
                            .ancestor_scope(scope, *distance)
                            .unwrap_or_else(|| {
                                panic!("Invalid ancestor scope for \"{}\"", name.lexeme_str())
                            })
                    }
                    None => GLOBAL_SCOPE,
                };
                state.env.assign(Some(scope), name.lexeme_str(), val.clone());
                Ok(val)
            }
            ExprKind::Logical {
                operator,
                left,
                right,
            } => match operator.kind {
                TokenKind::Or => {
                    let mut val = left.eval(state, scope)?;
                    if !val.is_truthy() {
                        val = right.eval(state, scope)?;
                    }
                    Ok(val)
                }
                TokenKind::And => {
                    let mut val = left.eval(state, scope)?;
                    if val.is_truthy() {
                        val = right.eval(state, scope)?;
                    }
                    Ok(val)
                }
                _ => Err(LoxError::Runtime(format!(
                    "Expected logical operator, got \"{}\"",
                    operator.lexeme_str()
                ))),
            },
            ExprKind::Call { callee, arguments } => {
                match callee.eval(state, scope)? {
                    LoxValue::Function(func) => {
                        func.borrow().call(state, scope, arguments)
                    },
                    LoxValue::Class(class) => {
                        let obj = Rc::new(RefCell::new(LoxObject {
                            class: class.clone(),
                            vars: LoxVars::new(),
                        }));
                        for (name, fun) in class.borrow().methods.iter() {
                            let mut method = fun.clone();
                            if matches!(&method.name, Some(name) if name == "init") {
                                method.is_constructor = true;
                            }
                            method.this = Some(obj.clone().into());
                            obj.borrow_mut().vars.insert(name.clone(), method.into());
                        }
                        let init = {
                            let obj = obj.borrow();
                            obj.vars.get("init")
                                .cloned()
                                .and_then(|init| init.get_fun().ok())
                        };
                        if let Some(init) = init {
                            init.borrow().call(state, scope, arguments)?;
                        }
                        Ok(obj.into())
                    },
                    _ => {
                        Err(LoxError::Runtime("Cannot call a non-function".into()))
                    }
                }
            },
            ExprKind::Get { left, right } => {
                let identifier = right.lexeme_str();
                let value = left.eval(state, scope)?
                        .get_object()?
                        .borrow()
                        .vars.get(&identifier)
                        .cloned()
                        .ok_or_else(|| LoxError::Runtime(format!("Undefined variable \"{}\"", identifier)))?;
                Ok(value)
            }
            ExprKind::Set { object, identifier, value } => {
                let obj = object.eval(state, scope)?.get_object()?;
                let val = value.eval(state, scope)?;
                obj.borrow_mut().vars.insert(identifier.lexeme_str(), val.clone());
                Ok(val)
            }
            ExprKind::This(..) => {
                let scope = match state.locals.get(&self._id) {
                    Some(depth) => state.env.ancestor_scope(scope, *depth),
                    None => Some(GLOBAL_SCOPE),
                };
                state.env.get(scope, "this").ok_or_else(|| LoxError::Runtime("Undefined variable \"this\"".into()))
            }
        }
    }
}

impl From<ExprKind> for Expr {
    fn from(value: ExprKind) -> Self {
        Expr::new(value)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Literal(value) => write!(f, "(literal {})", value.lexeme_str()),
            ExprKind::Unary { operator, right } => {
                write!(
                    f,
                    "({} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    right,
                )
            }
            ExprKind::Binary {
                operator,
                left,
                right,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    left,
                    right
                )
            }
            ExprKind::Grouping(inner) => {
                write!(f, "(grouping {})", inner)
            }
            ExprKind::Identifier(name) => {
                write!(f, "(identifier {})", name.lexeme_str())
            }
            ExprKind::Assignment { name, value } => {
                write!(f, "(= {} {})", name, value)
            }
            ExprKind::Logical {
                operator,
                left,
                right,
            } => {
                write!(
                    f,
                    "({} {} {})",
                    operator.lexeme.clone().unwrap_or("".to_owned()),
                    left,
                    right
                )
            }
            ExprKind::Call { callee, arguments } => {
                write!(
                    f,
                    "(call {} {})",
                    callee,
                    arguments
                        .iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            ExprKind::Get { left, right } => {
                write!(
                    f,
                    "(get {} {})",
                    left.to_string(),
                    right.lexeme_str()
                )
            }
            ExprKind::Set { object, identifier, value } => {
                write!(
                    f,
                    "(set (get {} {}) {})",
                    object.to_string(),
                    identifier.lexeme_str(),
                    value.to_string()
                )
            }
            ExprKind::This(..) => {
                write!(f, "(this)")
            }
        }
    }
}
