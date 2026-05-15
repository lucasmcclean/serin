use crate::lex::{self, token::Token};

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

pub struct Lexer<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            position: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Spanned<Token>>, lex::Error> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next()?;

            let is_eof = matches!(token.value, Token::Eof);

            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.position).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.source.get(self.position + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek();

        if byte.is_some() {
            self.position += 1;
        }

        byte
    }

    fn next(&mut self) -> Result<Spanned<Token>, lex::Error> {
        self.skip_whitespace()?;

        let start = self.position;

        let token = match self.peek() {
            Some(b'0'..=b'9') => self.lex_integer(),

            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => self.lex_identifier_or_keyword(),

            Some(b'\\') => {
                self.advance();
                Token::Backslash
            }

            Some(b'(') => {
                self.advance();
                Token::LeftParen
            }

            Some(b')') => {
                self.advance();
                Token::RightParen
            }

            Some(b',') => {
                self.advance();
                Token::Comma
            }

            Some(b'+') => {
                self.advance();
                Token::Plus
            }

            Some(b'*') => {
                self.advance();
                Token::Star
            }

            Some(b'/') => {
                self.advance();
                Token::Slash
            }

            Some(b'%') => {
                self.advance();
                Token::Percent
            }

            Some(b'=') => {
                self.advance();
                Token::Equals
            }

            Some(b'-') => {
                self.advance();

                if self.peek() == Some(b'>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }

            Some(b'<') => {
                self.advance();

                if self.peek() == Some(b'=') {
                    self.advance();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }

            Some(b'>') => {
                self.advance();

                if self.peek() == Some(b'=') {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }

            None => Token::Eof,

            Some(byte) => return Err(lex::Error::UnexpectedByte(byte, start)),
        };

        let end = self.position;

        Ok(Spanned {
            value: token,
            span: Span { start, end },
        })
    }

    fn lex_integer(&mut self) -> Token {
        let start = self.position;

        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.advance();
        }

        let slice = &self.source[start..self.position];

        let text = std::str::from_utf8(slice).unwrap();

        Token::Integer(text.parse().unwrap())
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;

        while matches!(
            self.peek(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        ) {
            self.advance();
        }

        let slice = &self.source[start..self.position];

        let text = std::str::from_utf8(slice).unwrap();

        match text {
            "let" => Token::Let,
            "in" => Token::In,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "not" => Token::Not,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),

            _ => Token::Identifier(text.to_string()),
        }
    }

    fn lex_comment(&mut self) -> Result<(), lex::Error> {
        self.advance();
        self.advance();

        let mut depth = 1;

        while let Some(byte) = self.peek() {
            match (byte, self.peek_next()) {
                (b'(', Some(b'*')) => {
                    self.advance();
                    self.advance();
                    depth += 1;
                }

                (b'*', Some(b')')) => {
                    self.advance();
                    self.advance();

                    depth -= 1;

                    if depth == 0 {
                        return Ok(());
                    }
                }

                _ => {
                    self.advance();
                }
            }
        }

        Err(lex::Error::UnterminatedComment)
    }

    fn skip_whitespace(&mut self) -> Result<(), lex::Error> {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\n' | b'\t' | b'\r')) {
                self.advance();
            }

            if self.peek() == Some(b'(') && self.peek_next() == Some(b'*') {
                self.lex_comment()?;
                continue;
            }

            break;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_integer() {
        let lexer = Lexer::new("123");

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].value, Token::Integer(123));
        assert_eq!(tokens[1].value, Token::Eof);
    }

    #[test]
    fn lex_identifier() {
        let lexer = Lexer::new("test");

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].value, Token::Identifier("test".to_string()));

        assert_eq!(tokens[1].value, Token::Eof);
    }

    #[test]
    fn lex_keywords() {
        let lexer = Lexer::new("let in if then else not");

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![
                Token::Let,
                Token::In,
                Token::If,
                Token::Then,
                Token::Else,
                Token::Not,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lex_booleans() {
        let lexer = Lexer::new("true false");

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![Token::Boolean(true), Token::Boolean(false), Token::Eof,]
        );
    }

    #[test]
    fn lex_lambda() {
        let lexer = Lexer::new(r"\x -> x");

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![
                Token::Backslash,
                Token::Identifier("x".into()),
                Token::Arrow,
                Token::Identifier("x".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lex_operators() {
        let lexer = Lexer::new("+ - * / % < > <= >=");

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Percent,
                Token::Less,
                Token::Greater,
                Token::LessEqual,
                Token::GreaterEqual,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lex_basic_program() {
        let source = r"
            let add = \x -> \y -> x + y
        ";

        let lexer = Lexer::new(source);

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![
                Token::Let,
                Token::Identifier("add".into()),
                Token::Equals,
                Token::Backslash,
                Token::Identifier("x".into()),
                Token::Arrow,
                Token::Backslash,
                Token::Identifier("y".into()),
                Token::Arrow,
                Token::Identifier("x".into()),
                Token::Plus,
                Token::Identifier("y".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lex_invalid_character() {
        let lexer = Lexer::new("@");

        let result = lexer.tokenize();

        assert!(result.is_err());
    }

    #[test]
    fn lex_comment() {
        let lexer = Lexer::new("123 (* comment *) 456");

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![Token::Integer(123), Token::Integer(456), Token::Eof,]
        );
    }

    #[test]
    fn lex_nested_comments() {
        let lexer = Lexer::new("123 (* outer (* inner *) outer *) 456");

        let tokens = lexer.tokenize().unwrap();

        let values: Vec<Token> = tokens.into_iter().map(|t| t.value).collect();

        assert_eq!(
            values,
            vec![Token::Integer(123), Token::Integer(456), Token::Eof,]
        );
    }

    #[test]
    fn lex_unterminated_comment() {
        let lexer = Lexer::new("(* test");

        let result = lexer.tokenize();

        assert!(matches!(result, Err(lex::Error::UnterminatedComment)));
    }
}
