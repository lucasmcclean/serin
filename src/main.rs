mod lex;

use std::{env, fs, process};

use lex::lexer::Lexer;

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

    match lexer.tokenize() {
        Ok(tokens) => {
            for token in tokens {
                println!(
                    "{:?} [{}..{}]",
                    token.value, token.span.start, token.span.end,
                );
            }
        }

        Err(error) => {
            eprintln!("lexer error: {:?}", error);
            process::exit(1);
        }
    }
}
