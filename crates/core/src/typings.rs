use crate::*;
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub literal: Option<ValueWrapper>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum ValueWrapper {
    Str(String),
    Num(f64),
    Bool(bool),
    Func(Rc<dyn Callable>),
    Nil,
}

impl PartialEq for ValueWrapper {
    fn eq(&self, other: &Self) -> bool {
        use ValueWrapper::*;
        match (self, other) {
            (Num(l0), Num(r0)) => l0 == r0,
            (Str(l0), Str(r0)) => l0 == r0,
            (Bool(l0), Bool(r0)) => l0 == r0,

            (Nil, Nil) => true,
            (Nil, _) => false,
            (_, Nil) => false,

            (Str(l0), Num(l1)) => l0 == &l1.to_string(),
            (Num(l0), Str(l1)) => &l0.to_string() == l1,

            (Num(l0), Bool(l1)) => l0 == &(*l1 as u8 as f64),
            (Bool(l0), Num(l1)) => &(*l0 as u8 as f64) == l1,

            _ => false,
            // (Bool(l0), Str(l1)) => &l0.to_string() == l1,
            // (Str(l0), Bool(l1)) => l0 == &l1.to_string(),
        }
    }
}

impl From<f64> for ValueWrapper {
    fn from(value: f64) -> Self {
        Self::Num(value)
    }
}

impl From<String> for ValueWrapper {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<bool> for ValueWrapper {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl ValueWrapper {
    pub fn try_into_num(&self) -> Self {
        match self {
            Self::Bool(x) => Self::Num(*x as u8 as f64),
            x => x.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,

    BangEqual,
    Bang,
    EqualEqual,
    Equal,
    LessEqual,
    Less,
    GreaterEqual,
    Greater,

    PlusEqual,
    MinusEqual,
    SlashEqual,
    StarEqual,

    Slash,
    NilLiteral,
    StringLiteral,
    NumberLiteral,
    BoolLiteral,
    Identifier,

    Continue,
    Break,
    And,
    Class,
    Else,
    Fn,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    EOF,
}

impl Token {
    pub fn new(ty: TokenType, lexeme: String, literal: Option<ValueWrapper>, line: usize) -> Self {
        Self {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.lexeme)
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl Display for ValueWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueWrapper::Str(s) => f.write_fmt(format_args!("{:?}", s)),
            ValueWrapper::Num(s) => f.write_fmt(format_args!("{:?}", s)),
            ValueWrapper::Bool(s) => f.write_fmt(format_args!("{:?}", s)),
            ValueWrapper::Nil => f.write_str("nil"),
            ValueWrapper::Func(_) => f.write_str("[<Func>]"),
        }
    }
}
