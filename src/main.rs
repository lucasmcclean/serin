mod ast;
mod lex;
mod parse;

use std::{env, fs, process};

use lex::lexer::Lexer;

use crate::parse::parser::Parser;

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

    for token in &tokens {
        println!(
            "{:?} [{}..{}]",
            token.value, token.span.start, token.span.end,
        );
    }

    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(expr) => {
            println!("{:#?}", expr);
        }
        Err(err) => {
            eprintln!("parse error: {:?}", err);
            process::exit(1);
        }
    }
}
