use thiserror::Error;

use crate::{Token, TokenType, ValueWrapper};
pub type Result<T> = std::result::Result<T, SolisError>;

#[derive(Error, Debug)]
pub enum SolisError {
    // Lexer
    #[error("unrecognizable character `{0}` was found.")]
    UnrecognizedCharacter(char),
    #[error("an unterminated string was found")]
    UnterminatedString,
    #[error("invalid floating-point literal `{0}` was found.")]
    InvalidNumber(String),

    // Parser
    #[error("`{0}` missing literal value")]
    MissingLiteral(TokenType),
    #[error("[line {} at `{}`] expected {expected}", token.line, token.lexeme)]
    MissingToken { token: Token, expected: String },

    // Interpreter
    #[error("[line {} at `{}`] invalid assignment target", token.line, token.lexeme)]
    InvalidAssignmentTarget { token: Token },
    #[error("[line {0}] {1}")]
    RuntimeError(usize, String),

    // Loop tools
    #[error("[line {} at `{}`] 'break' statement was called outside a loop.", location.line, location.lexeme)]
    Break { location: Token },
    #[error("[line {} at `{}`] 'continue' statement was called outside a loop.", location.line, location.lexeme)]
    Continue { location: Token },
    #[error("[line {} at `{}`] 'return' statement was called outside a block.", location.line, location.lexeme)]
    Return {
        location: Token,
        value: ValueWrapper,
    },
}
