use crate::lex::{Span, Token};

#[derive(Clone, Debug)]
pub enum Error {
    UnexpectedToken {
        expected: String,
        found: Token,
        span: Span,
    },

    Message {
        message: String,
        span: Span,
    },
}
