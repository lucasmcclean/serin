use crate::lex::{Span, Token};

#[derive(Clone, Debug)]
pub enum Error {
    UnexpectedToken {
        expected: String,
        found: Token,
        span: Span,
    },

    UnexpectedEof {
        expected: String,
    },

    Message {
        message: String,
        span: Span,
    },
}
