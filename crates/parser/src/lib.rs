#![allow(unused_assignments)]
use core::{TokenType::*, *};
use interpreter::functions::SolisFunction;

pub type ExprRes = Result<Box<Expr>>;
pub type StmtRes = Result<Box<Stmt>>;

macro_rules! stmt {
    ($self: expr, {$($ty: ident => $rs: ident $(;$args: expr)? ,)*}) => {
        if !$self.is_at_end() {
            match $self.peek().ty {
                $($ty => {$self.advance();return $self.$rs($($args,)*);})*
                _ => ()
            }
        }
    };
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // - Utils

    fn next_match_s(&mut self, ty: &TokenType) -> bool {
        if self.check(ty) {
            self.advance();
            return true;
        }
        false
    }

    fn next_match_m(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().ty == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        };
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().ty == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume<T: Into<String>>(&mut self, ty: TokenType, error: T) -> Result<Token> {
        if self.check(&ty) {
            return Ok(self.advance());
        };

        let peeked = self.peek();
        Err(SolisError::MissingToken {
            token: peeked,
            expected: error.into(),
        })
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().ty == Semicolon {
                return;
            };

            match self.peek().ty {
                Class | Fn | Var | For | If | While | Print | Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    // - End: Utils

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?)
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> StmtRes {
        if self.next_match_s(&Var) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> StmtRes {
        let name = self.consume(Identifier, "variable name")?;
        let mut inititalizer = None;
        if self.next_match_s(&Equal) {
            inititalizer = Some(self.expression()?);
        }
        self.consume(Semicolon, "`;` after expression.")?;

        Ok(Stmt::var(name, inititalizer))
    }

    fn statement(&mut self) -> StmtRes {
        stmt!(self, {
            Return => return_stmt,
            Fn => function; "function",
            Break => break_stmt,
            Continue => continue_stmt,
            For => for_stmt,
            While => while_stmt,
            If => if_stmt,
            Print => print_stmt,
            LeftBrace => block,
        });

        self.expression_stmt()
    }

    fn continue_stmt(&mut self) -> StmtRes {
        self.consume(Semicolon, "Expected ';' after continue statment.")?;
        Ok(Stmt::continuestmt(self.previous()))
    }

    fn break_stmt(&mut self) -> StmtRes {
        self.consume(Semicolon, "Expected ';' after break statment.")?;
        Ok(Stmt::breakstmt(self.previous()))
    }

    fn return_stmt(&mut self) -> StmtRes {
        let keyword = self.previous();
        let mut value = Expr::literal(ValueWrapper::Nil);
        if !self.check(&Semicolon) {
            value = self.expression()?;
        }
        self.consume(Semicolon, "Expected ';' after return value.")?;
        Ok(Stmt::returnstmt(keyword, value))
    }

    fn function(&mut self, kind: &str) -> StmtRes {
        let name = self.consume(Identifier, format!("Expected {} name.", kind))?;
        self.consume(LeftParen, format!("Expected '(' after {} name.", kind))?;
        let mut parameters = Vec::new();
        if !self.check(&RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(SolisError::RuntimeError(
                        self.peek().line,
                        String::from("Can't have more than 255 arguments."),
                    ));
                }

                parameters.push(self.consume(Identifier, "Expected parameter name.")?);
                if !self.next_match_s(&Comma) {
                    break;
                }
            }
        }
        self.consume(RightParen, String::from("Expected ')' after parameters."))?;

        self.consume(LeftBrace, format!("Expected '{{' before {} body.", kind))?;
        let body = self.block()?;
        Ok(Stmt::function(SolisFunction::new(name, parameters, body)))
    }

    fn for_stmt(&mut self) -> StmtRes {
        self.consume(LeftParen, "Expected '(' after 'while'.")?;
        let (mut initializer, mut condition, mut increment) = (None, None, None);

        // Initializer
        if self.next_match_s(&Semicolon) {
            initializer = None;
        } else if self.next_match_s(&Var) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_stmt()?);
        }

        // Condition
        if !self.check(&Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(Semicolon, "Expected ';' after loop condition.")?;

        // Increment
        if !self.check(&RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(RightParen, "Expected ')' after 'while'.")?;

        let mut body = self.statement()?;
        let condition = match condition {
            Some(t) => t,
            None => Expr::literal(ValueWrapper::Bool(true)),
        };

        if let Some(increment) = increment {
            body = Stmt::block(vec![*body, *Stmt::expression(increment)])
        }

        body = Stmt::whilestmt(condition, body);

        if let Some(initializer) = initializer {
            body = Stmt::block(vec![*initializer, *body])
        }

        Ok(body)
    }

    fn while_stmt(&mut self) -> StmtRes {
        self.consume(LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expected ')' after 'while'.")?;

        let body = self.statement()?;

        Ok(Stmt::whilestmt(condition, body))
    }

    fn if_stmt(&mut self) -> StmtRes {
        self.consume(LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expected ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = match self.next_match_s(&Else) {
            true => Some(self.statement()?),
            false => None,
        };

        Ok(Stmt::ifstmt(condition, then_branch, else_branch))
    }

    fn print_stmt(&mut self) -> StmtRes {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expected `;` after expression.")?;

        Ok(Stmt::print(expr))
    }

    fn block(&mut self) -> StmtRes {
        let mut statements = Vec::new();
        while !self.check(&RightBrace) {
            statements.push(*self.declaration()?);
        }

        self.consume(RightBrace, "Expected '}' after block.")?;
        Ok(Stmt::block(statements))
    }

    fn expression_stmt(&mut self) -> StmtRes {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expected `;` after expression.")?;

        Ok(Stmt::expression(expr))
    }

    fn expression(&mut self) -> ExprRes {
        self.assignment()
    }

    fn assignment(&mut self) -> ExprRes {
        let expr = self.or()?;

        if self.next_match_m(&[PlusEqual, MinusEqual, SlashEqual, StarEqual]) {
            let op = self.previous();
            let value = self.assignment()?;

            return match *expr {
                Expr::Variable { name } => Ok(Expr::assign(
                    name.clone(),
                    Expr::binary(
                        Expr::variable(name),
                        Token::new(
                            match op.ty {
                                PlusEqual => Plus,
                                MinusEqual => Minus,
                                SlashEqual => Slash,
                                StarEqual => Star,
                                _ => panic!("Unreachable"),
                            },
                            op.lexeme,
                            op.literal,
                            op.line,
                        ),
                        value,
                    ),
                )),
                _ => Err(SolisError::InvalidAssignmentTarget { token: op }),
            };
        }

        if self.next_match_s(&Equal) {
            let op = self.previous();
            let value = self.assignment()?;

            return match *expr {
                Expr::Variable { name } => Ok(Expr::assign(name, value)),
                _ => Err(SolisError::InvalidAssignmentTarget { token: op }),
            };
        }

        Ok(expr)
    }

    fn or(&mut self) -> ExprRes {
        let mut expr = self.and()?;

        while self.next_match_s(&Or) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::logical(expr, operator, right)
        }

        Ok(expr)
    }

    fn and(&mut self) -> ExprRes {
        let mut expr = self.equality()?;
        while self.next_match_s(&And) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::logical(expr, operator, right)
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ExprRes {
        let mut expr = self.comparison()?;

        while self.next_match_m(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ExprRes {
        let mut expr = self.term()?;

        while self.next_match_m(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> ExprRes {
        let mut expr = self.factor()?;

        while self.next_match_m(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ExprRes {
        let mut expr = self.unary()?;

        while self.next_match_m(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;

            expr = Expr::binary(expr, operator, right)
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ExprRes {
        if self.next_match_m(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;

            return Ok(Expr::unary(operator, right));
        }

        self.call()
    }

    fn call(&mut self) -> ExprRes {
        let mut expr = self.primary()?;
        loop {
            if self.next_match_s(&LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> ExprRes {
        let mut args = Vec::new();
        if !self.check(&RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err(SolisError::RuntimeError(
                        self.peek().line,
                        String::from("Can't have more than 255 arguments."),
                    ));
                }
                args.push(self.expression()?);
                if !self.next_match_s(&Comma) {
                    break;
                }
            }
        }

        let paren = self.consume(RightParen, "Expected ')' after arguments.")?;
        Ok(Expr::call(callee, paren, args))
    }

    fn primary(&mut self) -> ExprRes {
        if self.next_match_s(&BoolLiteral) {
            match self.previous().literal {
                Some(t) => return Ok(Expr::literal(t)),
                _ => return Err(SolisError::MissingLiteral(self.previous().ty)),
            }
        }

        if self.next_match_s(&NilLiteral) {
            return Ok(Expr::literal(ValueWrapper::Nil));
        }

        if self.next_match_m(&[StringLiteral, NumberLiteral]) {
            match self.previous().literal {
                Some(t) => return Ok(Expr::literal(t)),
                _ => return Err(SolisError::MissingLiteral(self.previous().ty)),
            }
        }
        if self.next_match_s(&Identifier) {
            return Ok(Expr::variable(self.previous()));
        }

        if self.next_match_s(&LeftParen) {
            let expr = self.expression()?;
            self.consume(RightParen, "`)`")?;

            return Ok(Expr::grouping(expr));
        }

        let pk = self.peek();
        Err(SolisError::MissingToken {
            token: pk,
            expected: "expression".into(),
        })
    }
}
