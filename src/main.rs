mod lexer;
mod parser;
mod compiler;
mod ast;
mod macro_expander;
mod transform;

use std::env;
use std::fs;
use std::process;
use transform::{TransformRegistry, EchoTransform};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input_file: Option<&String> = None;
    let mut transform_names: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--transforms" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --transforms requires an argument");
                    print_usage(&args[0]);
                    process::exit(1);
                }
                i += 1;
                transform_names = args[i].split(',').map(|s| s.trim().to_string()).collect();
            }
            arg if arg.starts_with("--") => {
                eprintln!("Error: unknown option '{}'", arg);
                print_usage(&args[0]);
                process::exit(1);
            }
            _ => {
                if input_file.is_some() {
                    eprintln!("Error: multiple input files specified");
                    print_usage(&args[0]);
                    process::exit(1);
                }
                input_file = Some(&args[i]);
            }
        }
        i += 1;
    }

    let input_file = match input_file {
        Some(f) => f,
        None => {
            eprintln!("Error: no input file specified");
            print_usage(&args[0]);
            process::exit(1);
        }
    };

    let source_code = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", input_file, err);
            process::exit(1);
        }
    };

    // Build transform registry from CLI args
    let mut registry = TransformRegistry::new();
    for name in &transform_names {
        match name.as_str() {
            "echo" => registry.register(Box::new(EchoTransform::new())),
            other => {
                eprintln!("Error: unknown transform '{}'", other);
                eprintln!("Available transforms: echo");
                process::exit(1);
            }
        }
    }

    match compile_lisp(&source_code, registry) {
        Ok(rust_code) => println!("{}", rust_code),
        Err(err) => {
            eprintln!("Compilation error: {}", err);
            process::exit(1);
        }
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [OPTIONS] <input.lisp>", program_name);
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --transforms <list>  Comma-separated list of transforms to apply");
    eprintln!("                       Available: echo");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  {} --transforms echo,optimization example.lisp", program_name);
}

fn compile_lisp(source: &str, registry: TransformRegistry) -> Result<String, String> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;

    // Apply AST transformations (between parsing and macro expansion)
    let mut transformed_ast = Vec::new();
    for mut expr in ast {
        registry.apply_all(&mut expr)
            .map_err(|e| format!("Transform error: {}", e))?;
        transformed_ast.push(expr);
    }

    // Expand macros in the transformed AST
    let mut expander = macro_expander::MacroExpander::new();
    let mut expanded_ast = Vec::new();

    for expr in transformed_ast {
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

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand to (* 5 2) and compile to Rust
        assert!(result.contains("(5 * 2)"));
    }

    #[test]
    fn test_pipeline_macro_with_multiple_params() {
        let source = r#"
            (defmacro add-and-mult (a b c) `(* (+ ,a ,b) ,c))
            (add-and-mult 1 2 3)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

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

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should fully expand nested macros to (* (* 5 2) 2)
        assert!(result.contains("((5 * 2) * 2)"));
    }

    #[test]
    fn test_pipeline_macro_with_regular_code() {
        let source = r#"
            (defmacro square (x) `(* ,x ,x))
            (+ (square 3) (square 4))
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand to (+ (* 3 3) (* 4 4))
        assert!(result.contains("((3 * 3) + (4 * 4))"));
    }

    #[test]
    fn test_pipeline_macro_depth_limit() {
        let source = r#"
            (defmacro infinite (x) `(infinite ,x))
            (infinite 1)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry);

        // Should error with max depth exceeded
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Maximum expansion depth"));
        assert!(error_msg.contains("infinite"));
    }

    #[test]
    fn test_pipeline_macro_parameter_mismatch() {
        let source = r#"
            (defmacro needs-two (a b) `(+ ,a ,b))
            (needs-two 1)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry);

        // Should error with parameter count mismatch
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parameter count mismatch"));
    }

    #[test]
    fn test_pipeline_ordering() {
        // This test verifies the pipeline ordering: parse → transform → expand → compile
        let source = r#"
            (defmacro when (condition body else-body) `(if ,condition ,body ,else-body))
            (when (> 5 3) (+ 1 2) (+ 4 5))
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

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

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand to (+ (+ 5 1) (- 10 1))
        assert!(result.contains("((5 + 1) + (10 - 1))"));
    }

    // Pattern matching tests

    #[test]
    fn test_pipeline_rest_parameter() {
        let source = r#"
            (defmacro add-all (first &rest rest) `(+ ,first ,@rest))
            (add-all 1 2 3 4 5)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand to (+ 1 2 3 4 5)
        assert!(result.contains("(1 + 2 + 3 + 4 + 5)"));
    }

    #[test]
    fn test_pipeline_rest_parameter_empty() {
        let source = r#"
            (defmacro add-all (first &rest rest) `(+ ,first ,@rest))
            (add-all 42)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand to (+ 42) which compiles to just 42
        assert!(result.contains("42"));
    }

    #[test]
    fn test_pipeline_rest_with_multiple_required() {
        let source = r#"
            (defmacro add-first-two-then-rest (a b &rest rest) `(+ (+ ,a ,b) ,@rest))
            (add-first-two-then-rest 1 2 3 4)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand to (+ (+ 1 2) 3 4)
        assert!(result.contains("((1 + 2) + 3 + 4)"));
    }

    #[test]
    fn test_pipeline_rest_too_few_args_error() {
        let source = r#"
            (defmacro needs-two (a b &rest rest) `(+ ,a ,b))
            (needs-two 1)
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry);

        // Should error - need at least 2 args but got only 1
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parameter count mismatch"));
    }

    #[test]
    fn test_pipeline_rest_complex_macro() {
        // Test a realistic macro using &rest
        let source = r#"
            (defmacro my-list (first &rest rest) `(list ,first ,@rest))
            (+ (my-list 1 2 3) (my-list 10 20))
        "#;

        let registry = TransformRegistry::new();
        let result = compile_lisp(source, registry).unwrap();

        // Should expand both my-list calls
        assert!(result.contains("vec![1, 2, 3]"));
        assert!(result.contains("vec![10, 20]"));
    }

    // Transform tests

    #[test]
    fn test_pipeline_with_echo_transform() {
        let source = r#"
            (+ 1 2)
        "#;

        let mut registry = TransformRegistry::new();
        registry.register(Box::new(EchoTransform::new()));
        let result = compile_lisp(source, registry).unwrap();

        // Echo transform should not affect output
        assert!(result.contains("(1 + 2)"));
    }

    #[test]
    fn test_pipeline_transform_preserves_semantics() {
        let source = r#"
            (defmacro double (x) `(* ,x 2))
            (double 21)
        "#;

        // Test with no transforms
        let registry1 = TransformRegistry::new();
        let result1 = compile_lisp(source, registry1).unwrap();

        // Test with echo transform
        let mut registry2 = TransformRegistry::new();
        registry2.register(Box::new(EchoTransform::new()));
        let result2 = compile_lisp(source, registry2).unwrap();

        // Results should be identical
        assert_eq!(result1, result2);
    }
}
