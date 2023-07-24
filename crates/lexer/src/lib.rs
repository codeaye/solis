mod macros;
use core::{typings::TokenType::*, *};
use log::debug;
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
    fn new(source: &'a str) -> Self {
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

    pub fn lex(source: &'a str) -> Result<Vec<Token>> {
        let mut slf = Self::new(source);
        let start = std::time::Instant::now();

        while let Some(curr) = slf.next() {
            slf.scan_token(curr)?
        }

        slf.add_token(TokenType::EOF, None);

        debug!("scanned {} lines in {:?}", slf.curr_line, start.elapsed());

        Ok(slf.result.clone())
    }

    fn scan_token(&mut self, curr: char) -> Result<()> {
        macro_rules! add {
            ($ty:ident) => {
                self.add_token($ty, None)
            };
        }

        macro_rules! add_op {
            ($fl:expr) => {
                paste::paste! {
                    self.peek_match('=', [<$fl Equal>], $fl)
                }
            };
        }

        match curr {
            // General characters
            '(' => add!(LeftParen),
            ')' => add!(RightParen),
            '{' => add!(LeftBrace),
            '}' => add!(RightBrace),
            ',' => add!(Comma),
            '.' => add!(Dot),
            ';' => add!(Semicolon),
            // Operators
            '!' => add_op!(Bang),
            '=' => add_op!(Equal),
            '<' => add_op!(Less),
            '>' => add_op!(Greater),
            // Compound Assignment Operators
            '+' => add_op!(Plus),
            '-' => add_op!(Minus),
            '*' => add_op!(Star),
            // Longer Lexemes
            '/' => self.handle_slash()?,
            '"' => self.handle_string()?,
            '0'..='9' => self.handle_number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.handle_identifier()?,
            // Other
            ' ' | '\r' | '\t' | '\n' => self.clear_buffer(),
            x => return Err(SolisError::UnrecognizedCharacter(x)),
        }

        Ok(())
    }

    fn handle_slash(&mut self) -> Result<()> {
        match self.peeked {
            Some(peeked) if peeked == '/' => {
                for x in self.by_ref() {
                    if x == '\n' {
                        break;
                    }
                }
                self.clear_buffer();
            }
            Some(peeked) if peeked == '*' => {
                while let Some(x) = self.next() {
                    if (x == '*') && (self.peeked == Some('/')) {
                        self.next();
                        break;
                    }
                }
                self.clear_buffer();
            }
            Some(peeked) if peeked == '=' => {
                self.next();
                self.add_token(SlashEqual, None)
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
            return Err(SolisError::UnterminatedString);
        }
        self.buffer = self.buffer[1..self.buffer.len() - 1].to_string();
        self.add_token(StringLiteral, Some(ValueWrapper::Str(self.buffer.clone())));
        Ok(())
    }

    fn handle_number(&mut self) -> Result<()> {
        while let Some(peeked) = self.peeked {
            if !matches!(peeked, '0'..='9' | '.') {
                break;
            }
            self.next();
        }
        self.add_token(
            NumberLiteral,
            Some(ValueWrapper::Num(self.buffer.parse::<f64>().map_err(
                |_| SolisError::InvalidNumber(self.buffer.clone()),
            )?)),
        );
        Ok(())
    }

    fn handle_identifier(&mut self) -> Result<()> {
        while let Some(peeked) = self.peeked {
            if !matches!(peeked, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                break;
            }
            self.next();
        }
        macros::identify!(
            self,
            [And, Class, Else, For, Fn, If, Or, Print, Return, Super, This, Var, While, Break, Continue],
            [
                "true" => (BoolLiteral, ValueWrapper::Bool(true)),
                "false" =>  (BoolLiteral, ValueWrapper::Bool(false)),
                "nil" =>  (NilLiteral, ValueWrapper::Nil)
            ],
            _ => self.add_token(Identifier, None)
        );
        Ok(())
    }
}
