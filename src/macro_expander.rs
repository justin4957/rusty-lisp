use crate::ast::LispExpr;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MacroExpander {
    macros: HashMap<String, MacroDefinition>,
    expansion_depth: usize,
    max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: LispExpr,
}

#[derive(Debug)]
pub enum MacroError {
    UndefinedMacro(String),
    ParameterCountMismatch { expected: usize, actual: usize },
    MaxDepthExceeded(usize),
    ExpansionError(String),
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MacroError::UndefinedMacro(name) => write!(f, "Undefined macro: {}", name),
            MacroError::ParameterCountMismatch { expected, actual } => {
                write!(f, "Parameter count mismatch: expected {}, got {}", expected, actual)
            }
            MacroError::MaxDepthExceeded(depth) => {
                write!(f, "Maximum expansion depth exceeded: {}", depth)
            }
            MacroError::ExpansionError(msg) => write!(f, "Expansion error: {}", msg),
        }
    }
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            expansion_depth: 0,
            max_depth: 100, // Prevent infinite recursion
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            macros: HashMap::new(),
            expansion_depth: 0,
            max_depth,
        }
    }

    /// Register a macro definition
    pub fn define_macro(&mut self, name: String, parameters: Vec<String>, body: LispExpr) {
        let definition = MacroDefinition {
            name: name.clone(),
            parameters,
            body,
        };
        self.macros.insert(name, definition);
    }

    /// Expand all macro calls in an expression recursively
    pub fn expand_all(&mut self, expr: LispExpr) -> Result<LispExpr, MacroError> {
        self.expansion_depth = 0;
        self.expand_expression(expr)
    }

    fn expand_expression(&mut self, expr: LispExpr) -> Result<LispExpr, MacroError> {

        match expr {
            // Handle macro definitions - add them to our registry
            LispExpr::Macro { name, parameters, body } => {
                self.define_macro(name.clone(), parameters, *body);
                // Macro definitions don't expand to anything in the output
                Ok(LispExpr::Nil)
            }

            // Handle macro calls - expand them (this is for backward compatibility)
            LispExpr::MacroCall { name, args } => {
                if self.expansion_depth > self.max_depth {
                    return Err(MacroError::MaxDepthExceeded(self.max_depth));
                }
                self.expansion_depth += 1;
                let result = self.expand_macro_call(&name, args);
                self.expansion_depth -= 1;
                result
            }

            // Handle lists - check if they're macro calls, otherwise expand recursively
            LispExpr::List(elements) => {
                if elements.is_empty() {
                    return Ok(LispExpr::List(elements));
                }

                // Check if this is a macro call (first element is a symbol that matches a macro)
                if let LispExpr::Symbol(name) = &elements[0] {
                    if self.macros.contains_key(name) {
                        // Check depth before expanding
                        if self.expansion_depth > self.max_depth {
                            return Err(MacroError::MaxDepthExceeded(self.max_depth));
                        }
                        
                        // This is a macro call - convert and expand
                        let args = elements[1..].to_vec();
                        self.expansion_depth += 1;
                        let result = self.expand_macro_call(name, args);
                        self.expansion_depth -= 1;
                        return result;
                    }
                }

                // Not a macro call - expand elements recursively
                let mut expanded_elements = Vec::new();
                for element in elements {
                    let expanded = self.expand_expression(element)?;
                    // Skip Nil expressions (from macro definitions)
                    if !matches!(expanded, LispExpr::Nil) {
                        expanded_elements.push(expanded);
                    }
                }
                Ok(LispExpr::List(expanded_elements))
            }

            // Handle quote family - these should not be expanded
            LispExpr::Quote(expr) => Ok(LispExpr::Quote(expr)),
            LispExpr::Quasiquote(expr) => {
                // Quasiquote requires special handling - expand unquotes but not the rest
                let expanded_inner = self.expand_quasiquote(*expr)?;
                Ok(LispExpr::Quasiquote(Box::new(expanded_inner)))
            }
            LispExpr::Unquote(expr) => {
                // Unquote should expand its contents
                let expanded = self.expand_expression(*expr)?;
                Ok(LispExpr::Unquote(Box::new(expanded)))
            }
            LispExpr::Splice(expr) => {
                // Splice should expand its contents
                let expanded = self.expand_expression(*expr)?;
                Ok(LispExpr::Splice(Box::new(expanded)))
            }

            // Atomic expressions don't need expansion
            _ => Ok(expr),
        }
    }

    fn expand_macro_call(&mut self, name: &str, args: Vec<LispExpr>) -> Result<LispExpr, MacroError> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| MacroError::UndefinedMacro(name.to_string()))?
            .clone();

        if macro_def.parameters.len() != args.len() {
            return Err(MacroError::ParameterCountMismatch {
                expected: macro_def.parameters.len(),
                actual: args.len(),
            });
        }

        // Create parameter bindings
        let mut bindings = HashMap::new();
        for (param, arg) in macro_def.parameters.iter().zip(args.iter()) {
            bindings.insert(param.clone(), arg.clone());
        }

        // Substitute parameters in the macro body
        let substituted_body = self.substitute_parameters(&macro_def.body, &bindings)?;

        // Recursively expand the result in case it contains more macro calls
        self.expand_expression(substituted_body)
    }

    fn expand_quasiquote(&mut self, expr: LispExpr) -> Result<LispExpr, MacroError> {
        match expr {
            LispExpr::Unquote(inner) => {
                // Expand the unquoted expression
                self.expand_expression(*inner)
            }
            LispExpr::List(elements) => {
                let mut expanded_elements = Vec::new();
                for element in elements {
                    if let LispExpr::Splice(splice_expr) = element {
                        // Handle splice - expand and flatten
                        let expanded = self.expand_expression(*splice_expr)?;
                        if let LispExpr::List(splice_elements) = expanded {
                            expanded_elements.extend(splice_elements);
                        } else {
                            return Err(MacroError::ExpansionError(
                                "Splice must expand to a list".to_string()
                            ));
                        }
                    } else {
                        let expanded = self.expand_quasiquote(element)?;
                        expanded_elements.push(expanded);
                    }
                }
                Ok(LispExpr::List(expanded_elements))
            }
            _ => Ok(expr),
        }
    }

    fn expand_quasiquote_with_substitution(
        &self,
        expr: LispExpr,
        bindings: &HashMap<String, LispExpr>,
    ) -> Result<LispExpr, MacroError> {
        match expr {
            LispExpr::Unquote(inner) => {
                // Unquoted expressions should be substituted directly
                self.substitute_parameters(&inner, bindings)
            }
            LispExpr::List(elements) => {
                let mut expanded_elements = Vec::new();
                for element in elements {
                    if let LispExpr::Splice(splice_expr) = element {
                        // Handle splice - substitute and flatten
                        let substituted = self.substitute_parameters(&splice_expr, bindings)?;
                        if let LispExpr::List(splice_elements) = substituted {
                            expanded_elements.extend(splice_elements);
                        } else {
                            return Err(MacroError::ExpansionError(
                                "Splice must expand to a list".to_string()
                            ));
                        }
                    } else {
                        let expanded = self.expand_quasiquote_with_substitution(element, bindings)?;
                        expanded_elements.push(expanded);
                    }
                }
                Ok(LispExpr::List(expanded_elements))
            }
            _ => Ok(expr),
        }
    }

    fn substitute_parameters(
        &self,
        expr: &LispExpr,
        bindings: &HashMap<String, LispExpr>,
    ) -> Result<LispExpr, MacroError> {
        match expr {
            LispExpr::Symbol(name) => {
                if let Some(replacement) = bindings.get(name) {
                    Ok(replacement.clone())
                } else {
                    Ok(expr.clone())
                }
            }
            LispExpr::List(elements) => {
                let mut substituted_elements = Vec::new();
                for element in elements {
                    substituted_elements.push(self.substitute_parameters(element, bindings)?);
                }
                Ok(LispExpr::List(substituted_elements))
            }
            LispExpr::Quote(inner) => {
                // Don't substitute inside quotes
                Ok(LispExpr::Quote(inner.clone()))
            }
            LispExpr::Quasiquote(inner) => {
                // Expand the quasiquote with substitutions
                let substituted = self.substitute_parameters(inner, bindings)?;
                self.expand_quasiquote_with_substitution(substituted, bindings)
            }
            LispExpr::Unquote(inner) => {
                // Substitute inside unquotes
                let substituted = self.substitute_parameters(inner, bindings)?;
                Ok(LispExpr::Unquote(Box::new(substituted)))
            }
            LispExpr::Splice(inner) => {
                // Substitute inside splices
                let substituted = self.substitute_parameters(inner, bindings)?;
                Ok(LispExpr::Splice(Box::new(substituted)))
            }
            // Other expressions are returned as-is
            _ => Ok(expr.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_macro_definition() {
        let mut expander = MacroExpander::new();
        
        // Define a simple macro: (defmacro double (x) `(* ,x 2))
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
                LispExpr::Number(2.0),
            ])
        ));
        
        expander.define_macro(
            "double".to_string(),
            vec!["x".to_string()],
            macro_body,
        );
        
        // Test macro call: (double 5)
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("double".to_string()),
            LispExpr::Number(5.0),
        ]);
        
        let result = expander.expand_all(macro_call).unwrap();
        
        // Should expand to: (* 5 2)
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("*".to_string()),
            LispExpr::Number(5.0),
            LispExpr::Number(2.0),
        ]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_macro_with_multiple_parameters() {
        let mut expander = MacroExpander::new();
        
        // Define macro: (defmacro add-and-multiply (a b c) `(* (+ ,a ,b) ,c))
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::List(vec![
                    LispExpr::Symbol("+".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("a".to_string()))),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("b".to_string()))),
                ]),
                LispExpr::Unquote(Box::new(LispExpr::Symbol("c".to_string()))),
            ])
        ));
        
        expander.define_macro(
            "add-and-multiply".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            macro_body,
        );
        
        // Test macro call: (add-and-multiply 1 2 3)
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("add-and-multiply".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
            LispExpr::Number(3.0),
        ]);
        
        let result = expander.expand_all(macro_call).unwrap();
        
        // Should expand to: (* (+ 1 2) 3)
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("*".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Number(1.0),
                LispExpr::Number(2.0),
            ]),
            LispExpr::Number(3.0),
        ]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_undefined_macro_no_error() {
        let mut expander = MacroExpander::new();
        
        // A function call that's not a macro should pass through unchanged
        let function_call = LispExpr::List(vec![
            LispExpr::Symbol("undefined_function".to_string()),
            LispExpr::Number(1.0),
        ]);
        
        let result = expander.expand_all(function_call.clone()).unwrap();
        
        // Should return the same expression unchanged
        assert_eq!(result, function_call);
    }

    #[test]
    fn test_parameter_count_mismatch() {
        let mut expander = MacroExpander::new();
        
        // Define macro with 2 parameters
        expander.define_macro(
            "test_macro".to_string(),
            vec!["a".to_string(), "b".to_string()],
            LispExpr::Symbol("body".to_string()),
        );
        
        // Call with wrong number of arguments
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("test_macro".to_string()),
            LispExpr::Number(1.0), // Only 1 argument
        ]);
        
        let result = expander.expand_all(macro_call);
        assert!(result.is_err());
        
        if let Err(MacroError::ParameterCountMismatch { expected, actual }) = result {
            assert_eq!(expected, 2);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected ParameterCountMismatch error");
        }
    }

    #[test]
    fn test_macro_definition_expansion() {
        let mut expander = MacroExpander::new();
        
        // Expand a macro definition - should register the macro and return Nil
        let macro_def = LispExpr::Macro {
            name: "test".to_string(),
            parameters: vec!["x".to_string()],
            body: Box::new(LispExpr::Symbol("x".to_string())),
        };
        
        let result = expander.expand_all(macro_def).unwrap();
        assert_eq!(result, LispExpr::Nil);
        
        // Verify the macro was registered
        assert!(expander.macros.contains_key("test"));
    }

    #[test]
    fn test_nested_macro_expansion() {
        let mut expander = MacroExpander::new();
        
        // Define first macro: (defmacro double (x) `(* ,x 2))
        expander.define_macro(
            "double".to_string(),
            vec!["x".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("*".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
                    LispExpr::Number(2.0),
                ])
            )),
        );
        
        // Define second macro that uses the first: (defmacro quadruple (x) `(double (double ,x)))
        expander.define_macro(
            "quadruple".to_string(),
            vec!["x".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("double".to_string()),
                    LispExpr::List(vec![
                        LispExpr::Symbol("double".to_string()),
                        LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
                    ]),
                ])
            )),
        );
        
        // This test shows that nested macro calls are properly expanded recursively
        
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("quadruple".to_string()),
            LispExpr::Number(5.0),
        ]);
        
        let result = expander.expand_all(macro_call).unwrap();
        
        // Should expand to: (* (* 5 2) 2) - quadruple means 4x, so (5 * 2) * 2 = 20
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("*".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::Number(5.0),
                LispExpr::Number(2.0),
            ]),
            LispExpr::Number(2.0),
        ]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_depth_exceeded() {
        let mut expander = MacroExpander::with_max_depth(2);
        
        // Define a recursive macro (infinite loop)
        // This should expand to a list that calls itself
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::List(vec![
                LispExpr::Symbol("recursive_macro".to_string()),
                LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
            ])
        ));
        
        expander.define_macro(
            "recursive_macro".to_string(),
            vec!["x".to_string()],
            macro_body,
        );
        
        // Use a List instead of MacroCall since that's what our parser produces
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("recursive_macro".to_string()),
            LispExpr::Number(1.0),
        ]);
        
        let result = expander.expand_all(macro_call);
        assert!(result.is_err());
        
        if let Err(MacroError::MaxDepthExceeded(depth)) = result {
            assert_eq!(depth, 2);
        } else {
            panic!("Expected MaxDepthExceeded error, got: {:?}", result);
        }
    }
}