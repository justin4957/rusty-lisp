mod lexer;
mod parser;
mod compiler;
mod ast;
mod macro_expander;

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
    
    // Expand macros in the AST
    let mut expander = macro_expander::MacroExpander::new();
    let mut expanded_ast = Vec::new();
    
    for expr in ast {
        let expanded = expander.expand_all(expr)
            .map_err(|e| format!("Macro expansion error: {}", e))?;
        
        // Skip Nil expressions (from macro definitions)
        if !matches!(expanded, ast::LispExpr::Nil) {
            expanded_ast.push(expanded);
        }
    }
    
    let rust_code = compiler::compile_to_rust(&expanded_ast)?;
    Ok(rust_code)
}
