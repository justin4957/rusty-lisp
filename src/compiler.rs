use crate::ast::LispExpr;

pub fn compile_to_rust(expressions: &[LispExpr]) -> Result<String, String> {
    let mut compiler = RustCompiler::new();
    
    let mut rust_code = String::new();
    rust_code.push_str("fn main() {\n");
    
    for expr in expressions {
        let compiled_expr = compiler.compile_expression(expr)?;
        rust_code.push_str(&format!("    println!(\"{{:?}}\", {});\n", compiled_expr));
    }
    
    rust_code.push_str("}\n");
    Ok(rust_code)
}

struct RustCompiler {
    variable_counter: usize,
}

impl RustCompiler {
    fn new() -> Self {
        Self {
            variable_counter: 0,
        }
    }
    
    fn next_temp_var(&mut self) -> String {
        let var = format!("temp_{}", self.variable_counter);
        self.variable_counter += 1;
        var
    }
    
    fn compile_expression(&mut self, expr: &LispExpr) -> Result<String, String> {
        match expr {
            LispExpr::Number(n) => Ok(n.to_string()),
            LispExpr::String(s) => Ok(format!("\"{}\"", s.replace('"', "\\\""))),
            LispExpr::Bool(b) => Ok(b.to_string()),
            LispExpr::Nil => Ok("()".to_string()),
            LispExpr::Symbol(s) => {
                match s.as_str() {
                    "pi" => Ok("std::f64::consts::PI".to_string()),
                    "e" => Ok("std::f64::consts::E".to_string()),
                    _ => Err(format!("Unknown symbol: {}", s))
                }
            },
            LispExpr::List(elements) => self.compile_list(elements),
            LispExpr::Macro { name, .. } => {
                Err(format!("Macro definitions are not yet supported in code generation: {}", name))
            },
            LispExpr::MacroCall { name, .. } => {
                Err(format!("Macro calls are not yet supported in code generation: {}", name))
            },
            LispExpr::Quote(_expr) => {
                Err("Quote expressions are not yet supported in code generation".to_string())
            },
            LispExpr::Quasiquote(_expr) => {
                Err("Quasiquote expressions are not yet supported in code generation".to_string())
            },
            LispExpr::Unquote(_expr) => {
                Err("Unquote expressions are not yet supported in code generation".to_string())
            },
            LispExpr::Splice(_expr) => {
                Err("Splice expressions are not yet supported in code generation".to_string())
            },
            LispExpr::Gensym(name) => {
                Ok(format!("generated_symbol_{}", name))
            },
        }
    }
    
    fn compile_list(&mut self, elements: &[LispExpr]) -> Result<String, String> {
        if elements.is_empty() {
            return Ok("vec![]".to_string());
        }
        
        let first = &elements[0];
        let args = &elements[1..];
        
        match first.as_symbol() {
            Some("+") => self.compile_arithmetic_op("+", args),
            Some("-") => self.compile_arithmetic_op("-", args),
            Some("*") => self.compile_arithmetic_op("*", args),
            Some("/") => self.compile_arithmetic_op("/", args),
            Some("=") => self.compile_comparison_op("==", args),
            Some("<") => self.compile_comparison_op("<", args),
            Some(">") => self.compile_comparison_op(">", args),
            Some("<=") => self.compile_comparison_op("<=", args),
            Some(">=") => self.compile_comparison_op(">=", args),
            Some("if") => self.compile_if(args),
            Some("let") => self.compile_let(args),
            Some("list") => self.compile_list_creation(args),
            Some(func_name) => Err(format!("Unknown function: {}", func_name)),
            None => Err("First element of list must be a symbol".to_string()),
        }
    }
    
    fn compile_arithmetic_op(&mut self, op: &str, args: &[LispExpr]) -> Result<String, String> {
        if args.is_empty() {
            return Err(format!("Arithmetic operation '{}' requires at least one argument", op));
        }
        
        let compiled_args: Result<Vec<String>, String> = args
            .iter()
            .map(|arg| self.compile_expression(arg))
            .collect();
        
        let compiled_args = compiled_args?;
        
        if compiled_args.len() == 1 {
            match op {
                "-" => Ok(format!("-({})", compiled_args[0])),
                _ => Ok(compiled_args[0].clone()),
            }
        } else {
            Ok(format!("({})", compiled_args.join(&format!(" {} ", op))))
        }
    }
    
    fn compile_comparison_op(&mut self, op: &str, args: &[LispExpr]) -> Result<String, String> {
        if args.len() != 2 {
            return Err(format!("Comparison operation '{}' requires exactly 2 arguments", op));
        }
        
        let left = self.compile_expression(&args[0])?;
        let right = self.compile_expression(&args[1])?;
        
        Ok(format!("({} {} {})", left, op, right))
    }
    
    fn compile_if(&mut self, args: &[LispExpr]) -> Result<String, String> {
        if args.len() != 3 {
            return Err("'if' requires exactly 3 arguments: condition, then-expr, else-expr".to_string());
        }
        
        let condition = self.compile_expression(&args[0])?;
        let then_expr = self.compile_expression(&args[1])?;
        let else_expr = self.compile_expression(&args[2])?;
        
        Ok(format!("if {} {{ {} }} else {{ {} }}", condition, then_expr, else_expr))
    }
    
    fn compile_let(&mut self, args: &[LispExpr]) -> Result<String, String> {
        if args.len() != 2 {
            return Err("'let' requires exactly 2 arguments: bindings and body".to_string());
        }
        
        let bindings = match &args[0] {
            LispExpr::List(bindings) => bindings,
            _ => return Err("First argument to 'let' must be a list of bindings".to_string()),
        };
        
        let mut rust_code = String::new();
        rust_code.push('{');
        
        for binding in bindings {
            match binding {
                LispExpr::List(binding_pair) if binding_pair.len() == 2 => {
                    let var_name = match &binding_pair[0] {
                        LispExpr::Symbol(name) => name,
                        _ => return Err("Variable name must be a symbol".to_string()),
                    };
                    let value = self.compile_expression(&binding_pair[1])?;
                    rust_code.push_str(&format!(" let {} = {};", var_name, value));
                },
                _ => return Err("Each binding must be a list of [variable, value]".to_string()),
            }
        }
        
        let body = self.compile_expression(&args[1])?;
        rust_code.push_str(&format!(" {} }}", body));
        
        Ok(rust_code)
    }
    
    fn compile_list_creation(&mut self, args: &[LispExpr]) -> Result<String, String> {
        let compiled_args: Result<Vec<String>, String> = args
            .iter()
            .map(|arg| self.compile_expression(arg))
            .collect();
        
        let compiled_args = compiled_args?;
        Ok(format!("vec![{}]", compiled_args.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    
    #[test]
    fn test_compile_arithmetic() {
        let tokens = tokenize("(+ 1 2 3)").unwrap();
        let ast = parse(tokens).unwrap();
        let rust_code = compile_to_rust(&ast).unwrap();
        
        assert!(rust_code.contains("(1 + 2 + 3)"));
    }
    
    #[test]
    fn test_compile_nested_expression() {
        let tokens = tokenize("(* (+ 1 2) 3)").unwrap();
        let ast = parse(tokens).unwrap();
        let rust_code = compile_to_rust(&ast).unwrap();
        
        assert!(rust_code.contains("((1 + 2) * 3)"));
    }
}