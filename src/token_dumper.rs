use std::env;
use std::fs;

mod lexer;
mod ast;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: token_dumper <file>");
        return;
    }

    let file_path = &args[1];
    let source = fs::read_to_string(file_path).expect("Failed to read file");

    let mut lexer = lexer::Lexer::new(source, file_path.to_string());
    match lexer.tokenize() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => {
            eprintln!("Lexer error: {}", e);
        }
    }
}
