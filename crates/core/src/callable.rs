use crate::*;
use std::fmt::{Debug, Display};

pub trait Callable {
    fn name(&self) -> &str;
    fn arity(&self) -> usize;
    fn call(&self, arguments: Vec<ValueWrapper>, env: EnvData) -> Result<ValueWrapper>;
}

impl Display for dyn Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[<callable (\"{}\")>]", self.name()))
    }
}

impl Debug for dyn Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[<callable (\"{}\")>]", self.name()))
    }
}

// Native functions

pub struct NativeFuncClock;
impl Callable for NativeFuncClock {
    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> usize {
        0
    }
    fn call(&self, _arguments: Vec<ValueWrapper>, _env: EnvData) -> Result<ValueWrapper> {
        if let Ok(n) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(ValueWrapper::Num(n.as_secs_f64()))
        } else {
            panic!("SystemTime before UNIX_EPOCH.");
        }
    }
}
