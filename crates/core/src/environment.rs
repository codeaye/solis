use crate::*;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, collections::hash_map::Entry, rc::Rc};

pub type RcCell<T> = Rc<RefCell<T>>;
pub type EnvData = RcCell<Environment>;

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<RcCell<Environment>>,
    values: FxHashMap<String, ValueWrapper>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut globals = FxHashMap::default();
        globals.insert(
            String::from("clock"),
            ValueWrapper::Func(Rc::new(NativeFuncClock {})),
        );

        Rc::new(RefCell::new(Self {
            values: globals,
            enclosing: None,
        }))
    }

    pub fn new_with_enclosing(enclosing: RcCell<Environment>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: FxHashMap::default(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, key: String, value: ValueWrapper) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: String, line: usize) -> Result<ValueWrapper> {
        if let Some(value) = self.values.get(&key) {
            return Ok(value.clone());
        };

        if let Some(enclosing) = &self.enclosing {
            let value = enclosing.borrow().get(key, line);
            return value;
        }

        Err(SolisError::RuntimeError(
            line,
            format!("Unexpected variable name `{}`.", key),
        ))
    }

    pub fn assign(&mut self, key: String, value: ValueWrapper, line: usize) -> Result<()> {
        if let Entry::Occupied(mut e) = self.values.entry(key.clone()) {
            e.insert(value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            let value = enclosing.borrow_mut().assign(key, value, line);
            return value;
        }

        Err(SolisError::RuntimeError(
            line,
            format!("Undefined variable `{}`.", key),
        ))
    }
}
