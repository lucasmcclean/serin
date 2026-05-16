mod ast;
mod lex;
mod parse;

use std::{env, fs, process};

use lex::lexer::Lexer;

use crate::{ast::Expression, parse::parser::Parser};

fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: serin <file.sn>");
            process::exit(1);
        }
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read file '{}': {}", path, error);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&source);

    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,

        Err(error) => {
            eprintln!("lexer error: {:?}", error);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(expr) => print!("{}", expr.pretty()),
        Err(err) => {
            eprintln!("parse error: {:?}", err);
            process::exit(1);
        }
    }
}

impl Expression {
    pub fn pretty(&self) -> String {
        let mut out = String::new();
        self.pretty_with_indent(0, &mut out);
        out
    }

    fn pretty_with_indent(&self, indent: usize, out: &mut String) {
        let pad = "  ".repeat(indent);

        match self {
            Expression::Integer(n) => {
                out.push_str(&format!("{pad}Integer({n})\n"));
            }

            Expression::Boolean(b) => {
                out.push_str(&format!("{pad}Boolean({b})\n"));
            }

            Expression::Identifier(name) => {
                out.push_str(&format!("{pad}Identifier(\"{name}\")\n"));
            }

            Expression::Unary { operator, operand } => {
                out.push_str(&format!("{pad}Unary({operator:?})\n"));
                operand.pretty_with_indent(indent + 1, out);
            }

            Expression::Binary {
                operator,
                left,
                right,
            } => {
                out.push_str(&format!("{pad}{operator:?}\n"));
                left.pretty_with_indent(indent + 1, out);
                right.pretty_with_indent(indent + 1, out);
            }

            Expression::Application { function, argument } => {
                out.push_str(&format!("{pad}Application\n"));
                function.pretty_with_indent(indent + 1, out);
                argument.pretty_with_indent(indent + 1, out);
            }

            Expression::Let { name, value, body } => {
                out.push_str(&format!("{pad}Let(\"{name}\")\n"));
                value.pretty_with_indent(indent + 1, out);
                body.pretty_with_indent(indent + 1, out);
            }

            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                out.push_str(&format!("{pad}If\n"));
                condition.pretty_with_indent(indent + 1, out);
                then_branch.pretty_with_indent(indent + 1, out);
                else_branch.pretty_with_indent(indent + 1, out);
            }

            Expression::Lambda {
                parameter,
                annotation,
                body,
            } => {
                out.push_str(&format!("{pad}Lambda(\"{parameter}\")\n"));

                if let Some(t) = annotation {
                    out.push_str(&format!("{pad}  Type: {:?}\n", t));
                }

                body.pretty_with_indent(indent + 1, out);
            }

            Expression::Tuple(items) => {
                out.push_str(&format!("{pad}Tuple\n"));

                for item in items {
                    item.pretty_with_indent(indent + 1, out);
                }
            }
        }
    }
}
