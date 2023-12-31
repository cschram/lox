use super::{
    environment::{ScopeHandle, GLOBAL_SCOPE},
    error::*,
    object::*,
    scanner::{Token, TokenKind},
    state::LoxState,
    value::LoxValue,
};
use std::{
    cell::RefCell,
    cmp::{Ord, Ordering},
    fmt,
    hash::{Hash, Hasher},
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
    Super(Token),
}

#[derive(PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    _id: usize,
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self._id.hash(state);
    }
}

impl Eq for Expr {}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self._id.cmp(&other._id))
    }
}

impl Ord for Expr {
    fn cmp(&self, other: &Self) -> Ordering {
        self._id.cmp(&other._id)
    }
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

    pub fn line(&self) -> u32 {
        match &self.kind {
            ExprKind::Literal(token) => token.line,
            ExprKind::Unary { operator, .. } => operator.line,
            ExprKind::Binary { operator, .. } => operator.line,
            ExprKind::Grouping(expr) => expr.line(),
            ExprKind::Identifier(token) => token.line,
            ExprKind::Assignment { name, .. } => name.line,
            ExprKind::Logical { operator, .. } => operator.line,
            ExprKind::Call { callee, .. } => callee.line(),
            ExprKind::Get { left, .. } => left.line(),
            ExprKind::Set { object, .. } => object.line(),
            ExprKind::This(token) => token.line,
            ExprKind::Super(token) => token.line,
        }
    }

    pub fn eval(&self, state: &mut LoxState, scope: ScopeHandle) -> LoxResult<LoxValue> {
        // println!("{self}");
        match &self.kind {
            ExprKind::Literal(value) => Ok(LoxValue::from(value.clone())),
            ExprKind::Unary { operator, right } => match operator.kind {
                TokenKind::Bang => {
                    // let right_value = self.evaluate_expr(scope, right)?.is_truthy();
                    let right_value = right.eval(state, scope)?.is_truthy();
                    Ok(LoxValue::Boolean(!right_value))
                }
                _ => Err(LoxError::Runtime(
                    format!("Unknown unary operator \"{}\"", operator),
                    self.line(),
                )),
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
                                left_value.get_number(self.line())?
                                    + right_value.get_number(self.line())?,
                            ))
                        } else {
                            Err(LoxError::Runtime(
                                format!(
                                    "Invalid operands {} + {}",
                                    left_value.to_string(),
                                    right_value.to_string(),
                                ),
                                self.line(),
                            ))
                        }
                    }
                    TokenKind::Minus => Ok(LoxValue::Number(
                        left_value.get_number(self.line())?
                            - right_value.get_number(self.line())?,
                    )),
                    TokenKind::Star => Ok(LoxValue::Number(
                        left_value.get_number(self.line())?
                            * right_value.get_number(self.line())?,
                    )),
                    TokenKind::Slash => Ok(LoxValue::Number(
                        left_value.get_number(self.line())?
                            / right_value.get_number(self.line())?,
                    )),
                    TokenKind::Greater => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number(self.line())?
                                    > right_value.get_number(self.line())?,
                            ))
                        } else {
                            Err(LoxError::Runtime(
                                format!(
                                    "Invalid operands {} > {}",
                                    left_value.to_string(),
                                    right_value.to_string(),
                                ),
                                self.line(),
                            ))
                        }
                    }
                    TokenKind::GreaterEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number(self.line())?
                                    >= right_value.get_number(self.line())?,
                            ))
                        } else {
                            Err(LoxError::Runtime(
                                format!(
                                    "Invalid operands {} >= {}",
                                    left_value.to_string(),
                                    right_value.to_string(),
                                ),
                                self.line(),
                            ))
                        }
                    }
                    TokenKind::Less => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number(self.line())?
                                    < right_value.get_number(self.line())?,
                            ))
                        } else {
                            Err(LoxError::Runtime(
                                format!(
                                    "Invalid operands {} < {}",
                                    left_value.to_string(),
                                    right_value.to_string(),
                                ),
                                self.line(),
                            ))
                        }
                    }
                    TokenKind::LessEqual => {
                        if left_value.is_number() && right_value.is_number() {
                            Ok(LoxValue::Boolean(
                                left_value.get_number(self.line())?
                                    <= right_value.get_number(self.line())?,
                            ))
                        } else {
                            Err(LoxError::Runtime(
                                format!(
                                    "Invalid operands {} <= {}",
                                    left_value.to_string(),
                                    right_value.to_string(),
                                ),
                                self.line(),
                            ))
                        }
                    }
                    TokenKind::EqualEqual => Ok(LoxValue::Boolean(left_value == right_value)),
                    TokenKind::BangEqual => Ok(LoxValue::Boolean(left_value != right_value)),
                    _ => Err(LoxError::Runtime(
                        format!("Unknown binary operator \"{}\"", operator),
                        self.line(),
                    )),
                }
            }
            ExprKind::Grouping(inner) => inner.eval(state, scope),
            ExprKind::Identifier(name) => {
                state.resolve_local(scope, self, &name.lexeme_str(), self.line())
            }
            ExprKind::Assignment { name, value } => {
                let val = value.eval(state, scope)?;
                let scope =
                    match state.locals.get(self) {
                        Some(distance) => state
                            .env
                            .ancestor_scope(scope, *distance)
                            .unwrap_or_else(|| {
                                panic!("Invalid ancestor scope for \"{}\"", name.lexeme_str())
                            }),
                        None => GLOBAL_SCOPE,
                    };
                state
                    .env
                    .assign(Some(scope), name.lexeme_str(), val.clone());
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
                _ => Err(LoxError::Runtime(
                    format!(
                        "Expected logical operator, got \"{}\"",
                        operator.lexeme_str()
                    ),
                    self.line(),
                )),
            },
            ExprKind::Call { callee, arguments } => match callee.eval(state, scope)? {
                LoxValue::Function(func) => {
                    func.borrow().call(state, scope, arguments, self.line())
                }
                LoxValue::Class(class) => Ok(LoxObject::instantiate(
                    class,
                    state,
                    scope,
                    arguments,
                    self.line(),
                )?),
                _ => Err(LoxError::Runtime(
                    "Cannot call a non-function".into(),
                    self.line(),
                )),
            },
            ExprKind::Get { left, right } => {
                let identifier = right.lexeme_str();
                let value = left
                    .eval(state, scope)?
                    .get_object(self.line())?
                    .borrow()
                    .get(&identifier)
                    .ok_or_else(|| {
                        LoxError::Runtime(
                            format!("Undefined variable \"{}\"", identifier),
                            self.line(),
                        )
                    })?;
                Ok(value)
            }
            ExprKind::Set {
                object,
                identifier,
                value,
            } => {
                let obj = object.eval(state, scope)?.get_object(self.line())?;
                let val = value.eval(state, scope)?;
                obj.borrow_mut().set(identifier.lexeme_str(), val.clone());
                Ok(val)
            }
            ExprKind::This(_) => state.resolve_local(scope, self, "this", self.line()),
            ExprKind::Super(method) => {
                let super_value = state
                    .resolve_local(scope, self, "super", self.line())?
                    .get_super(self.line())?;
                super_value
                    .get(&method.lexeme_str())
                    .cloned()
                    .ok_or_else(|| {
                        LoxError::Runtime(
                            format!("Undefined super method \"{}\"", method.lexeme_str()),
                            self.line(),
                        )
                    })
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
                write!(f, "(get {} {})", left, right.lexeme_str())
            }
            ExprKind::Set {
                object,
                identifier,
                value,
            } => {
                write!(
                    f,
                    "(set (get {} {}) {})",
                    object,
                    identifier.lexeme_str(),
                    value
                )
            }
            ExprKind::This(_) => {
                write!(f, "(this)")
            }
            ExprKind::Super(method) => {
                write!(f, "(super {})", method.lexeme_str())
            }
        }
    }
}
