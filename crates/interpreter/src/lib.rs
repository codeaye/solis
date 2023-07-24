#![feature(if_let_guard)]
pub mod functions;

use core::{TokenType::*, ValueWrapper::*, *};

pub trait InterpretStmt {
    fn evaluate(&self, env: EnvData) -> Result<()>;
}

pub trait InterpretExpr {
    fn evaluate(&self, env: EnvData) -> Result<ValueWrapper>;
    fn evaluate_binary(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: EnvData,
    ) -> Result<ValueWrapper>;
    fn evaluate_unary(&self, operator: &Token, right: &Expr, env: EnvData) -> Result<ValueWrapper>;
    fn evaluate_literal(&self, val: &ValueWrapper, env: EnvData) -> Result<ValueWrapper>;
    fn evaluate_grouping(&self, expr: &Expr, env: EnvData) -> Result<ValueWrapper>;
}

impl InterpretStmt for Stmt {
    fn evaluate(&self, env: EnvData) -> Result<()> {
        match self {
            Stmt::Expression { expression } => {
                expression.evaluate(env)?;
            }
            Stmt::Print { expression } => {
                println!("{}", expression.evaluate(env)?);
            }
            Stmt::Var { name, inititalizer } => {
                if let Some(initalizer) = inititalizer {
                    let val = initalizer.evaluate(env.clone())?;
                    env.borrow_mut().define(name.to_string(), val);
                }
            }
            Stmt::Block { statements } => {
                let env = Environment::new_with_enclosing(env);
                for statement in statements {
                    statement.evaluate(env.clone())?;
                }
                drop(env)
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                if condition.evaluate(env.clone())? == true.into() {
                    then_branch.evaluate(env)?;
                } else if let Some(else_branch) = else_branch {
                    else_branch.evaluate(env)?;
                }
            }
            Stmt::WhileStmt { condition, body } => {
                while condition.evaluate(env.clone())? == Bool(true) {
                    match body.evaluate(env.clone()) {
                        Err(SolisError::Break { .. }) => break,
                        Err(SolisError::Continue { .. }) => continue,
                        Err(e) => return Err(e),
                        _ => (),
                    }
                }
            }
            Stmt::BreakStmt { location } => {
                return Err(SolisError::Break {
                    location: location.clone(),
                })
            }
            Stmt::ContinueStmt { location } => {
                return Err(SolisError::Continue {
                    location: location.clone(),
                })
            }
            Stmt::Function { instance } => env
                .borrow_mut()
                .define(instance.name().to_string(), Func(instance.clone())),
            Stmt::ReturnStmt { keyword, value } => {
                let value = value.evaluate(env)?;
                return Err(SolisError::Return {
                    location: keyword.clone(),
                    value,
                });
            }
        }

        Ok(())
    }
}

impl InterpretExpr for Expr {
    fn evaluate_binary(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: EnvData,
    ) -> Result<ValueWrapper> {
        let left = left.evaluate(env.clone())?;
        let right = right.evaluate(env)?;

        match operator.ty {
            BangEqual => return Ok((!(left == right)).into()),
            EqualEqual => return Ok((left == right).into()),
            _ => (),
        }

        match (&left.try_into_num(), &right.try_into_num()) {
            (Num(l), Num(r)) => match &operator.ty {
                Greater => return Ok((l > r).into()),
                GreaterEqual => return Ok((l >= r).into()),
                Less => return Ok((l < r).into()),
                LessEqual => return Ok((l <= r).into()),

                Minus => return Ok((l - r).into()),
                Star => return Ok((l * r).into()),
                Slash => return Ok((l / r).into()),
                Plus => return Ok((l + r).into()),
                _ => (),
            },
            (Str(left), Str(right)) if operator.ty == Plus => {
                return Ok((left.clone() + right).into())
            }
            (Num(left), Str(right)) if operator.ty == Plus => {
                return Ok((left.to_string() + right).into())
            }
            (Str(left), Num(right)) if operator.ty == Plus => {
                return Ok((left.to_owned() + &right.to_string()).into())
            }
            _ => (),
        }

        Err(SolisError::RuntimeError(
            operator.line,
            format!("Invalid equation: {:?} `{}` {:?}", left, operator, right),
        ))
    }

    fn evaluate_grouping(&self, expr: &Expr, env: EnvData) -> Result<ValueWrapper> {
        expr.evaluate(env)
    }

    fn evaluate_literal(&self, val: &ValueWrapper, _env: EnvData) -> Result<ValueWrapper> {
        Ok(val.clone())
    }

    fn evaluate_unary(&self, operator: &Token, right: &Expr, env: EnvData) -> Result<ValueWrapper> {
        let right = right.evaluate(env)?;
        match operator.ty {
            Minus if let Num(x) = &right => {
                Ok((*x).into())
            },
            Bang => {
                Ok(match right {
                    Bool(x) => !x,
                    Nil => false,
                    _ => true
                }.into())
            }
            _ => Err(SolisError::RuntimeError(operator.line, format!("Unrecognized operator `{}` with value `{}`.", operator, right), )),
        }
    }

    fn evaluate(&self, env: EnvData) -> Result<ValueWrapper> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right, env),
            Expr::Grouping { expression } => self.evaluate_grouping(expression, env),
            Expr::Literal { value } => self.evaluate_literal(value, env),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(env.clone())?;

                if operator.ty == Or {
                    if left == Bool(true) {
                        return Ok(left);
                    };
                } else if left == Bool(false) {
                    return Ok(left);
                }

                right.evaluate(env)
            }
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right, env),
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                let callee = callee.evaluate(env.clone())?;
                let mut arguments = Vec::with_capacity(args.capacity());
                for argument in args {
                    arguments.push(argument.evaluate(env.clone())?)
                }

                let Func(function) = callee else {
                    return Err(SolisError::RuntimeError(paren.line, String::from("Can only call functions and classes.")))
                };

                if arguments.len() != function.arity() {
                    return Err(SolisError::RuntimeError(
                        paren.line,
                        format!(
                            "Expected {} arguments but recieved {} arguments.",
                            function.arity(),
                            arguments.len()
                        ),
                    ));
                }

                function.call(arguments, env)
            }
            Expr::Variable { name } => env.borrow().get(name.lexeme.clone(), name.line),
            Expr::Assign { name, value } => {
                let value = value.evaluate(env.clone())?;
                env.borrow_mut()
                    .assign(name.lexeme.clone(), value.clone(), name.line)?;
                Ok(value)
            }
        }
    }
}
pub struct Interpreter {
    environment: EnvData,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&self, statements: Vec<Box<Stmt>>) -> Result<()> {
        for statement in statements {
            statement.evaluate(self.environment.clone())?;
        }
        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
