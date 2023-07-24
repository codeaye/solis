use crate::InterpretStmt;
use core::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct SolisFunction {
    pub name: String,
    params: Vec<Token>,
    body: Box<Stmt>,
}
impl SolisFunction {
    pub fn new(name: Token, params: Vec<Token>, body: Box<Stmt>) -> Rc<Self> {
        Rc::new(SolisFunction {
            name: name.lexeme,
            params,
            body,
        })
    }
}
impl Callable for SolisFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, arguments: Vec<ValueWrapper>, env: EnvData) -> Result<ValueWrapper> {
        let environment = Environment::new_with_enclosing(env);
        for (i, item) in arguments.iter().enumerate() {
            environment
                .borrow_mut()
                .define(self.params[i].lexeme.clone(), item.clone());
        }

        let output = self.body.evaluate(environment);
        match output {
            Ok(_) => Ok(ValueWrapper::Nil),
            Err(SolisError::Return { location: _, value }) => Ok(value),
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
