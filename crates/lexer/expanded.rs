#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use core::{typings::TokenType::*, *};
use std::{iter::Peekable, str::Chars};
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    buffer: String,
    result: Vec<Token>,
    current: Option<char>,
    peeked: Option<char>,
    curr_line: usize,
}
impl Iterator for Lexer<'_> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let (curr, peek) = (self.source.next(), self.source.peek().cloned());
        if curr == Some('\n') {
            self.curr_line += 1;
        }
        self.current = curr;
        self.peeked = peek;
        if let Some(current) = curr {
            self.buffer.push(current);
        }
        curr
    }
}
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            buffer: String::with_capacity(10),
            result: Vec::new(),
            current: None,
            peeked: None,
            curr_line: 1,
        }
    }
    fn peek_match(&mut self, to_match: char, tr: TokenType, fl: TokenType) {
        let ty = match self.peeked {
            Some(peeked) if peeked == to_match => {
                self.next();
                tr
            }
            _ => fl,
        };
        self.add_token(ty, None);
    }
    fn clear_buffer(&mut self) {
        self.buffer.clear();
    }
    fn add_token(&mut self, token_ty: TokenType, literal: Option<ValueWrapper>) {
        self.result.push(Token::new(
            token_ty,
            self.buffer.clone(),
            literal,
            self.curr_line,
        ));
        self.clear_buffer()
    }
    pub fn lex(&mut self) -> Result<Vec<Token>> {
        while let Some(curr) = self.next() {
            self.scan_token(curr)?
        }
        Ok(self.result.clone())
    }
    fn scan_token(&mut self, curr: char) -> Result<()> {
        match curr {
            '(' => self.add_token(LeftParen, None),
            ')' => self.add_token(RightParen, None),
            '{' => self.add_token(LeftBrace, None),
            '}' => self.add_token(RightBrace, None),
            ',' => self.add_token(Comma, None),
            '.' => self.add_token(Dot, None),
            '-' => self.add_token(Minus, None),
            '+' => self.add_token(Plus, None),
            ';' => self.add_token(Semicolon, None),
            '*' => self.add_token(Star, None),
            '!' => self.peek_match('=', BangEqual, Bang),
            '=' => self.peek_match('=', EqualEqual, Equal),
            '<' => self.peek_match('=', LessEqual, Less),
            '>' => self.peek_match('=', GreaterEqual, Greater),
            '/' => self.handle_slash()?,
            '"' => self.handle_string()?,
            '0'..='9' => self.handle_number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.handle_identifier()?,
            ' ' | '\r' | '\t' => self.clear_buffer(),
            x => self.clear_buffer(),
        }
        Ok(())
    }
    fn handle_slash(&mut self) -> Result<()> {
        match self.peeked {
            Some(peeked) if peeked == '/' => {
                while let Some(x) = self.next() {
                    if x == '\n' {
                        break;
                    }
                }
                self.clear_buffer();
            }
            Some(peeked) if peeked == '*' => {
                while let Some(x) = self.next() {
                    if (x == '*') && (self.peeked == Some('/')) {
                        break;
                    }
                }
                self.clear_buffer();
            }
            _ => self.add_token(Slash, None),
        }
        Ok(())
    }
    fn handle_string(&mut self) -> Result<()> {
        let mut terminated = false;
        while let Some(peeked) = self.peeked {
            self.next();
            if peeked == '"' {
                terminated = true;
                break;
            }
        }
        if !terminated {
            return Err(HarmonyError::UnterminatedString);
        }
        self.buffer = self.buffer[1..self.buffer.len() - 1].to_string();
        self.add_token(StringLiteral, Some(ValueWrapper::Str(self.buffer.clone())));
        Ok(())
    }
    fn handle_number(&mut self) -> Result<()> {
        while let Some(peeked) = self.peeked {
            if !match peeked {
                '0'..='9' | '.' => true,
                _ => false,
            } {
                break;
            }
            self.next();
        }
        self.add_token(
            NumberLiteral,
            Some(ValueWrapper::Num(self.buffer.parse::<f64>().map_err(
                |_| HarmonyError::InvalidNumber(self.buffer.clone()),
            )?)),
        );
        Ok(())
    }
    fn handle_identifier(&mut self) -> Result<()> {
        while let Some(peeked) = self.peeked {
            if !match peeked {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
                _ => false,
            } {
                break;
            }
            self.next();
        }
        match self.buffer.as_str() {
            "and" => self.add_token(TokenType::And, None),
            "class" => self.add_token(TokenType::Class, None),
            "else" => self.add_token(TokenType::Else, None),
            "for" => self.add_token(TokenType::For, None),
            "fun" => self.add_token(TokenType::Fun, None),
            "if" => self.add_token(TokenType::If, None),
            "or" => self.add_token(TokenType::Or, None),
            "print" => self.add_token(TokenType::Print, None),
            "return" => self.add_token(TokenType::Return, None),
            "super" => self.add_token(TokenType::Super, None),
            "this" => self.add_token(TokenType::This, None),
            "var" => self.add_token(TokenType::Var, None),
            "while" => self.add_token(TokenType::While, None),
            "true" => self.add_token(BooleanLiteral, Some(ValueWrapper::Bool(true))),
            "false" => self.add_token(BooleanLiteral, Some(ValueWrapper::Bool(false))),
            "nil" => self.add_token(NilLiteral, Some(ValueWrapper::Nil)),
            _ => self.add_token(Identifier, None),
        };
        Ok(())
    }
}
