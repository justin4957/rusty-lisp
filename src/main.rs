mod lexer;
mod parser;
mod compiler;
mod ast;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <input.lisp>", args[0]);
        process::exit(1);
    }
    
    let input_file = &args[1];
    let source_code = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", input_file, err);
            process::exit(1);
        }
    };
    
    match compile_lisp(&source_code) {
        Ok(rust_code) => println!("{}", rust_code),
        Err(err) => {
            eprintln!("Compilation error: {}", err);
            process::exit(1);
        }
    }
}

fn compile_lisp(source: &str) -> Result<String, String> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;
    let rust_code = compiler::compile_to_rust(&ast)?;
    Ok(rust_code)
}
