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

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;

        while let Token::EqualEqual = self.peek().value {
            let operator = BinaryOperator::Equal;

            self.advance();
            let right = self.parse_comparison()?;

            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_addition()?;

        loop {
            let operator = match self.peek().value {
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_addition()?;

            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplication()?;

        loop {
            let operator = match self.peek().value {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplication()?;

            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
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

    fn expect_integer(expr: Expression, expected: i64) {
        match expr {
            Expression::Integer(value) => assert_eq!(value, expected),
            other => panic!("expected integer {expected}, got {other:?}"),
        }
    }

    fn expect_boolean(expr: Expression, expected: bool) {
        match expr {
            Expression::Boolean(value) => assert_eq!(value, expected),
            other => panic!("expected boolean {expected}, got {other:?}"),
        }
    }

    fn expect_identifier(expr: Expression, expected: &str) {
        match expr {
            Expression::Identifier(name) => assert_eq!(name, expected),
            other => panic!("expected identifier {expected:?}, got {other:?}"),
        }
    }

    fn expect_unary(expr: Expression, expected: UnaryOperator) -> Expression {
        match expr {
            Expression::Unary { operator, operand } => {
                assert_eq!(
                    std::mem::discriminant(&operator),
                    std::mem::discriminant(&expected),
                    "expected unary operator {:?}, got {:?}",
                    expected,
                    operator
                );
                *operand
            }
            other => panic!("expected unary {:?}, got {other:?}", expected),
        }
    }

    fn expect_binary(expr: Expression, expected: BinaryOperator) -> (Expression, Expression) {
        match expr {
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                assert_eq!(
                    std::mem::discriminant(&operator),
                    std::mem::discriminant(&expected),
                    "expected binary operator {:?}, got {:?}",
                    expected,
                    operator
                );
                (*left, *right)
            }
            other => panic!("expected binary {:?}, got {other:?}", expected),
        }
    }

    fn expect_application(expr: Expression) -> (Expression, Expression) {
        match expr {
            Expression::Application { function, argument } => (*function, *argument),
            other => panic!("expected application, got {other:?}"),
        }
    }

    #[test]
    fn parses_integer() {
        let expr = parse_source("42").unwrap();
        expect_integer(expr, 42);
    }

    #[test]
    fn parses_boolean_true() {
        let expr = parse_source("true").unwrap();
        expect_boolean(expr, true);
    }

    #[test]
    fn parses_boolean_false() {
        let expr = parse_source("false").unwrap();
        expect_boolean(expr, false);
    }

    #[test]
    fn parses_identifier() {
        let expr = parse_source("x").unwrap();
        expect_identifier(expr, "x");
    }

    #[test]
    fn parses_parenthesized_integer() {
        let expr = parse_source("(42)").unwrap();
        expect_integer(expr, 42);
    }

    #[test]
    fn parses_parenthesized_identifier() {
        let expr = parse_source("(x)").unwrap();
        expect_identifier(expr, "x");
    }

    #[test]
    fn parses_nested_parentheses() {
        let expr = parse_source("((x))").unwrap();
        expect_identifier(expr, "x");
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
    fn parses_application_associativity() {
        let expr = parse_source("f x y").unwrap();

        let (fx, y) = expect_application(expr);
        let (f, x) = expect_application(fx);

        expect_identifier(f, "f");
        expect_identifier(x, "x");
        expect_identifier(y, "y");
    }

    #[test]
    fn parses_application_with_parenthesized_function() {
        let expr = parse_source("(f x) y").unwrap();

        let (fx, y) = expect_application(expr);
        let (f, x) = expect_application(fx);

        expect_identifier(f, "f");
        expect_identifier(x, "x");
        expect_identifier(y, "y");
    }

    #[test]
    fn parses_application_with_parenthesized_argument() {
        let expr = parse_source("f (x + y)").unwrap();

        let (f, arg) = expect_application(expr);
        expect_identifier(f, "f");

        let (x, y) = expect_binary(arg, BinaryOperator::Add);
        expect_identifier(x, "x");
        expect_identifier(y, "y");
    }

    #[test]
    fn parses_unary_negation() {
        let expr = parse_source("-x").unwrap();
        let inner = expect_unary(expr, UnaryOperator::Negate);
        expect_identifier(inner, "x");
    }

    #[test]
    fn parses_unary_not() {
        let expr = parse_source("not x").unwrap();
        let inner = expect_unary(expr, UnaryOperator::Not);
        expect_identifier(inner, "x");
    }

    #[test]
    fn parses_double_unary_nesting() {
        let expr = parse_source("--x").unwrap();
        let inner = expect_unary(expr, UnaryOperator::Negate);
        let inner = expect_unary(inner, UnaryOperator::Negate);
        expect_identifier(inner, "x");
    }

    #[test]
    fn unary_binds_to_application() {
        let expr = parse_source("-f x").unwrap();

        let inner = expect_unary(expr, UnaryOperator::Negate);
        let (f, x) = expect_application(inner);

        expect_identifier(f, "f");
        expect_identifier(x, "x");
    }

    #[test]
    fn unary_binds_tighter_than_multiplication_on_left() {
        let expr = parse_source("-a * b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Multiply);
        let inner = expect_unary(left, UnaryOperator::Negate);

        expect_identifier(inner, "a");
        expect_identifier(right, "b");
    }

    #[test]
    fn unary_binds_tighter_than_multiplication_on_right() {
        let expr = parse_source("a * -b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Multiply);

        expect_identifier(left, "a");
        let inner = expect_unary(right, UnaryOperator::Negate);
        expect_identifier(inner, "b");
    }

    #[test]
    fn multiplication_is_left_associative() {
        let expr = parse_source("a * b * c").unwrap();

        let (left, c) = expect_binary(expr, BinaryOperator::Multiply);
        let (a, b) = expect_binary(left, BinaryOperator::Multiply);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn division_and_modulo_parse_at_multiplicative_precedence() {
        let expr = parse_source("a / b % c").unwrap();

        let (left, c) = expect_binary(expr, BinaryOperator::Modulo);
        let (a, b) = expect_binary(left, BinaryOperator::Divide);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        let expr = parse_source("a + b * c").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Add);
        let (b, c) = expect_binary(right, BinaryOperator::Multiply);

        expect_identifier(left, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn addition_is_left_associative() {
        let expr = parse_source("a + b + c").unwrap();

        let (left, c) = expect_binary(expr, BinaryOperator::Add);
        let (a, b) = expect_binary(left, BinaryOperator::Add);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn subtraction_is_left_associative() {
        let expr = parse_source("a - b - c").unwrap();

        let (left, c) = expect_binary(expr, BinaryOperator::Subtract);
        let (a, b) = expect_binary(left, BinaryOperator::Subtract);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn x_minus_y_is_binary_subtraction() {
        let expr = parse_source("x -y").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Subtract);
        expect_identifier(left, "x");
        expect_identifier(right, "y");
    }

    #[test]
    fn x_minus_minus_y_is_subtraction_with_unary_rhs() {
        let expr = parse_source("x - -y").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Subtract);
        expect_identifier(left, "x");

        let inner = expect_unary(right, UnaryOperator::Negate);
        expect_identifier(inner, "y");
    }

    #[test]
    fn parentheses_override_precedence() {
        let expr = parse_source("(a + b) * c").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Multiply);
        let (a, b) = expect_binary(left, BinaryOperator::Add);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(right, "c");
    }

    #[test]
    fn deeply_nested_parentheses_override_precedence() {
        let expr = parse_source("((a + b) * (c - d))").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Multiply);
        let (a, b) = expect_binary(left, BinaryOperator::Add);
        let (c, d) = expect_binary(right, BinaryOperator::Subtract);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
        expect_identifier(d, "d");
    }

    #[test]
    fn parses_equality() {
        let expr = parse_source("a == b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Equal);
        expect_identifier(left, "a");
        expect_identifier(right, "b");
    }

    #[test]
    fn equality_is_left_associative() {
        let expr = parse_source("a == b == c").unwrap();

        let (left, c) = expect_binary(expr, BinaryOperator::Equal);
        let (a, b) = expect_binary(left, BinaryOperator::Equal);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn comparison_binds_tighter_than_equality() {
        let expr = parse_source("a < b == c").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Equal);
        let (a, b) = expect_binary(left, BinaryOperator::Less);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(right, "c");
    }

    #[test]
    fn parses_less_than() {
        let expr = parse_source("a < b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Less);
        expect_identifier(left, "a");
        expect_identifier(right, "b");
    }

    #[test]
    fn parses_less_equal() {
        let expr = parse_source("a <= b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::LessEqual);
        expect_identifier(left, "a");
        expect_identifier(right, "b");
    }

    #[test]
    fn parses_greater_than() {
        let expr = parse_source("a > b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Greater);
        expect_identifier(left, "a");
        expect_identifier(right, "b");
    }

    #[test]
    fn parses_greater_equal() {
        let expr = parse_source("a >= b").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::GreaterEqual);
        expect_identifier(left, "a");
        expect_identifier(right, "b");
    }

    #[test]
    fn comparison_is_left_associative() {
        let expr = parse_source("a < b < c").unwrap();

        let (left, c) = expect_binary(expr, BinaryOperator::Less);
        let (a, b) = expect_binary(left, BinaryOperator::Less);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(c, "c");
    }

    #[test]
    fn addition_binds_tighter_than_comparison() {
        let expr = parse_source("a + b < c").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Less);
        let (a, b) = expect_binary(left, BinaryOperator::Add);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(right, "c");
    }

    #[test]
    fn addition_binds_tighter_than_equality() {
        let expr = parse_source("a + b == c").unwrap();

        let (left, right) = expect_binary(expr, BinaryOperator::Equal);
        let (a, b) = expect_binary(left, BinaryOperator::Add);

        expect_identifier(a, "a");
        expect_identifier(b, "b");
        expect_identifier(right, "c");
    }

    #[test]
    fn complex_precedence_chain_parses_correctly() {
        let expr = parse_source("f x + -a * b == c < d").unwrap();

        let (left_eq, right_less) = expect_binary(expr, BinaryOperator::Equal);

        let (add_left, add_right) = expect_binary(left_eq, BinaryOperator::Add);
        let (f, x) = expect_application(add_left);
        expect_identifier(f, "f");
        expect_identifier(x, "x");

        let (mul_left, mul_right) = expect_binary(add_right, BinaryOperator::Multiply);
        let a = expect_unary(mul_left, UnaryOperator::Negate);
        expect_identifier(a, "a");
        expect_identifier(mul_right, "b");

        let (c, d) = expect_binary(right_less, BinaryOperator::Less);
        expect_identifier(c, "c");
        expect_identifier(d, "d");
    }
}
