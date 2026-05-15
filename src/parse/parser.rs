use crate::{
    ast::expression::{BinaryOperator, Expression, UnaryOperator},
    lex::{lexer::Spanned, token::Token},
    parse,
};

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    position: usize,
}

pub type Result<T> = std::result::Result<T, parse::Error>;

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Expression> {
        let expr = self.parse_expression()?;

        match &self.peek().value {
            Token::Eof => Ok(expr),
            token => Err(parse::Error::UnexpectedToken {
                expected: "end of input".into(),
                found: token.clone(),
                span: self.peek().span,
            }),
        }
    }

    fn peek(&self) -> &Spanned<Token> {
        &self.tokens[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, kind: &Token) -> Result<()> {
        let token = self.peek();

        if matches!(token.value, Token::Eof) {
            return Err(parse::Error::UnexpectedEof {
                expected: format!("{:?}", kind),
            });
        }

        if std::mem::discriminant(&token.value) == std::mem::discriminant(kind) {
            self.advance();
            Ok(())
        } else {
            Err(parse::Error::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: token.value.clone(),
                span: token.span,
            })
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_multiplication()
    }

    fn parse_multiplication(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary()?;

        loop {
            let operator = match self.peek().value {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => break,
            };

            self.advance();
            let right = self.parse_unary()?;

            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        match self.peek().value {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(expr),
                })
            }

            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    operator: UnaryOperator::Not,
                    operand: Box::new(expr),
                })
            }

            _ => self.parse_application(),
        }
    }

    fn parse_application(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        while self.starts_primary() {
            let arg = self.parse_primary()?;

            expr = Expression::Application {
                function: Box::new(expr),
                argument: Box::new(arg),
            };
        }

        Ok(expr)
    }

    fn starts_primary(&self) -> bool {
        matches!(
            &self.peek().value,
            Token::Integer(_) | Token::Boolean(_) | Token::Identifier(_) | Token::LeftParen
        )
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match &self.peek().value {
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Integer(n))
            }

            Token::Boolean(b) => {
                let b = *b;
                self.advance();
                Ok(Expression::Boolean(b))
            }

            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expression::Identifier(name))
            }

            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                Ok(expr)
            }

            Token::Eof => Err(parse::Error::UnexpectedEof {
                expected: "expression".into(),
            }),

            _ => Err(parse::Error::UnexpectedToken {
                expected: "primary expression".into(),
                found: self.peek().value.clone(),
                span: self.peek().span,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(source: &str) -> Result<Expression> {
        let lexer = crate::lex::lexer::Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn parses_integer() {
        let expr = parse_source("42").unwrap();

        assert!(matches!(expr, Expression::Integer(42)));
    }

    #[test]
    fn parses_boolean() {
        let expr = parse_source("true").unwrap();

        assert!(matches!(expr, Expression::Boolean(true)));
    }

    #[test]
    fn parses_identifier() {
        let expr = parse_source("x").unwrap();

        assert!(matches!(expr, Expression::Identifier(ref name) if name == "x"));
    }

    #[test]
    fn parses_parenthesized_integer() {
        let expr = parse_source("(42)").unwrap();

        assert!(matches!(expr, Expression::Integer(42)));
    }

    #[test]
    fn parses_parenthesized_identifier() {
        let expr = parse_source("(x)").unwrap();

        assert!(matches!(expr, Expression::Identifier(ref name) if name == "x"));
    }

    #[test]
    fn parses_nested_parentheses() {
        let expr = parse_source("((x))").unwrap();

        assert!(matches!(expr, Expression::Identifier(ref name) if name == "x"));
    }

    #[test]
    fn errors_on_unclosed_paren() {
        let err = parse_source("(x").unwrap_err();

        match err {
            parse::Error::UnexpectedEof { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parses_application_chain() {
        let expr = parse_source("f x y").unwrap();

        match expr {
            Expression::Application { function, .. } => match *function {
                Expression::Application { .. } => {}
                Expression::Identifier(_) => {
                    panic!("expected left-associative application chain")
                }
                _ => panic!("unexpected structure"),
            },
            _ => panic!("expected application"),
        }
    }

    #[test]
    fn parses_unary_negation() {
        let expr = parse_source("-x").unwrap();

        match expr {
            Expression::Unary { .. } => {}
            _ => panic!("expected unary"),
        }
    }

    #[test]
    fn parses_double_unary() {
        let expr = parse_source("--x").unwrap();

        match expr {
            Expression::Unary { .. } => {}
            _ => panic!("expected unary chain"),
        }
    }

    #[test]
    fn parses_multiplication() {
        let expr = parse_source("a * b").unwrap();

        match expr {
            Expression::Binary { .. } => {}
            _ => panic!("expected binary"),
        }
    }

    #[test]
    fn parses_multiplication_chain() {
        let expr = parse_source("a * b * c").unwrap();

        match expr {
            Expression::Binary { .. } => {}
            _ => panic!("expected left-associative chain"),
        }
    }

    #[test]
    fn parses_mixed_unary_and_multiplication() {
        let expr = parse_source("-a * b").unwrap();

        match expr {
            Expression::Binary { .. } => {}
            _ => panic!("expected binary"),
        }
    }
}
