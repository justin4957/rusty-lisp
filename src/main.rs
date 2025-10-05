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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_simple_macro() {
        let source = r#"
            (defmacro double (x) `(* ,x 2))
            (double 5)
        "#;

        let result = compile_lisp(source).unwrap();

        // Should expand to (* 5 2) and compile to Rust
        assert!(result.contains("(5 * 2)"));
    }

    #[test]
    fn test_pipeline_macro_with_multiple_params() {
        let source = r#"
            (defmacro add-and-mult (a b c) `(* (+ ,a ,b) ,c))
            (add-and-mult 1 2 3)
        "#;

        let result = compile_lisp(source).unwrap();

        // Should expand to (* (+ 1 2) 3)
        assert!(result.contains("((1 + 2) * 3)"));
    }

    #[test]
    fn test_pipeline_nested_macros() {
        let source = r#"
            (defmacro double (x) `(* ,x 2))
            (defmacro quadruple (x) `(double (double ,x)))
            (quadruple 5)
        "#;

        let result = compile_lisp(source).unwrap();

        // Should fully expand nested macros to (* (* 5 2) 2)
        assert!(result.contains("((5 * 2) * 2)"));
    }

    #[test]
    fn test_pipeline_macro_with_regular_code() {
        let source = r#"
            (defmacro square (x) `(* ,x ,x))
            (+ (square 3) (square 4))
        "#;

        let result = compile_lisp(source).unwrap();

        // Should expand to (+ (* 3 3) (* 4 4))
        assert!(result.contains("((3 * 3) + (4 * 4))"));
    }

    #[test]
    fn test_pipeline_macro_depth_limit() {
        let source = r#"
            (defmacro infinite (x) `(infinite ,x))
            (infinite 1)
        "#;

        let result = compile_lisp(source);

        // Should error with max depth exceeded
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum expansion depth exceeded"));
    }

    #[test]
    fn test_pipeline_macro_parameter_mismatch() {
        let source = r#"
            (defmacro needs-two (a b) `(+ ,a ,b))
            (needs-two 1)
        "#;

        let result = compile_lisp(source);

        // Should error with parameter count mismatch
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parameter count mismatch"));
    }

    #[test]
    fn test_pipeline_ordering() {
        // This test verifies the pipeline ordering: parse → expand → compile
        let source = r#"
            (defmacro when (condition body else-body) `(if ,condition ,body ,else-body))
            (when (> 5 3) (+ 1 2) (+ 4 5))
        "#;

        let result = compile_lisp(source).unwrap();

        // Should expand when macro to if expression
        assert!(result.contains("if"));
        assert!(result.contains("(5 > 3)"));
        assert!(result.contains("(1 + 2)"));
    }

    #[test]
    fn test_pipeline_multiple_macros() {
        // Test that multiple macro definitions and uses work correctly
        let source = r#"
            (defmacro inc (x) `(+ ,x 1))
            (defmacro dec (x) `(- ,x 1))
            (+ (inc 5) (dec 10))
        "#;

        let result = compile_lisp(source).unwrap();

        // Should expand to (+ (+ 5 1) (- 10 1))
        assert!(result.contains("((5 + 1) + (10 - 1))"));
    }
}
