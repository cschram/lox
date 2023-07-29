use super::{environment::*, error::*, expr::Expr, scanner::*, state::LoxState, stmt::*, value::*};

pub type NativeFunction = fn(Vec<LoxValue>) -> LoxResult<LoxValue>;

#[derive(PartialEq, Clone)]
pub enum FunctionBody {
    Block(Vec<Stmt>, ScopeHandle),
    Native(NativeFunction),
}

#[derive(PartialEq, Clone)]
pub struct LoxFunction {
    pub name: Option<String>,
    pub params: Vec<Token>,
    pub body: FunctionBody,
    pub this_value: Option<LoxValue>,
    pub super_value: Option<LoxValue>,
    pub is_constructor: bool,
}

impl LoxFunction {
    pub fn from_stmt(stmt: &Stmt, scope: ScopeHandle) -> LoxResult<Self> {
        if let Stmt::Fun { name, params, body } = stmt {
            let identifier = name.lexeme_str();
            Ok(LoxFunction {
                name: Some(identifier.clone()),
                params: params.clone(),
                body: FunctionBody::Block(body.clone(), scope),
                this_value: None,
                super_value: None,
                is_constructor: false,
            })
        } else {
            Err(LoxError::Runtime("Expected a function statement".into()))
        }
    }

    pub fn native(name: &str, params: Vec<&str>, body: NativeFunction) -> Self {
        LoxFunction {
            name: Some(name.into()),
            params: params
                .into_iter()
                .map(|param| Token::new(TokenKind::Identifier, Some(param.into()), None, 0))
                .collect(),
            body: FunctionBody::Native(body),
            this_value: None,
            super_value: None,
            is_constructor: false,
        }
    }

    pub fn call(
        &self,
        state: &mut LoxState,
        scope: ScopeHandle,
        arguments: &[Expr],
    ) -> LoxResult<LoxValue> {
        if arguments.len() != self.params.len() {
            Err(LoxError::Runtime(format!(
                "Function \"{}\" takes {} argument(s)",
                self.name.clone().unwrap_or("".into()),
                self.params.len(),
            )))
        } else {
            // Evaluate arguments to get their final value
            let mut args: Vec<LoxValue> = vec![];
            for arg in arguments.iter() {
                args.push(arg.eval(state, scope)?);
            }
            let return_value = match &self.body {
                FunctionBody::Block(statements, closure) => {
                    // Bind arguments
                    for (i, arg) in args.drain(0..).enumerate() {
                        state
                            .env
                            .declare(Some(*closure), self.params[i].lexeme_str(), arg);
                    }
                    // Bind this value
                    let ret_value = if let Some(this) = &self.this_value {
                        state
                            .env
                            .declare(Some(*closure), "this".into(), this.clone());
                        if self.is_constructor {
                            this.clone()
                        } else {
                            LoxValue::Nil
                        }
                    } else {
                        LoxValue::Nil
                    };
                    // Execute function body
                    state.stack.push(ret_value);
                    for stmt in statements.iter() {
                        stmt.eval(state, *closure)?;
                        if matches!(stmt, Stmt::Return(_)) {
                            break;
                        }
                    }
                    state.stack.pop().unwrap()
                }
                FunctionBody::Native(func) => func(args)?,
            };
            Ok(return_value)
        }
    }
}