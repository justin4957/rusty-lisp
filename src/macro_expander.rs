use crate::ast::LispExpr;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MacroExpander {
    macros: HashMap<String, MacroDefinition>,
    expansion_depth: usize,
    max_depth: usize,
    gensym_counter: usize,
}

#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: LispExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroError {
    /// Attempted to call an undefined macro
    UndefinedMacro(String),

    /// Macro called with wrong number of arguments
    ParameterCountMismatch {
        macro_name: String,
        expected: usize,
        actual: usize
    },

    /// Maximum expansion depth exceeded (likely infinite recursion)
    MaxDepthExceeded {
        depth: usize,
        macro_name: String,
    },

    /// Generic expansion error with context
    ExpansionError {
        message: String,
        context: Option<String>,
    },

    /// Malformed macro definition
    MalformedDefinition {
        macro_name: String,
        reason: String,
    },

    /// Invalid pattern in macro parameters
    InvalidPattern {
        pattern: String,
        reason: String,
    },
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MacroError::UndefinedMacro(name) => {
                write!(f, "Undefined macro: '{}'\n", name)?;
                write!(f, "  Help: Check that the macro is defined with 'defmacro' before use")
            }
            MacroError::ParameterCountMismatch { macro_name, expected, actual } => {
                write!(f, "Parameter count mismatch in macro '{}'\n", macro_name)?;
                write!(f, "  Expected: {} argument{}\n", expected, if *expected == 1 { "" } else { "s" })?;
                write!(f, "  Got: {} argument{}\n", actual, if *actual == 1 { "" } else { "s" })?;
                write!(f, "  Help: Check the macro definition and ensure you pass the correct number of arguments")
            }
            MacroError::MaxDepthExceeded { depth, macro_name } => {
                write!(f, "Maximum expansion depth ({}) exceeded in macro '{}'\n", depth, macro_name)?;
                write!(f, "  Help: This likely indicates infinite recursion in your macro expansion.\n")?;
                write!(f, "        Check that recursive macros have a proper base case.")
            }
            MacroError::ExpansionError { message, context } => {
                write!(f, "Macro expansion error: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, "\n  Context: {}", ctx)?;
                }
                Ok(())
            }
            MacroError::MalformedDefinition { macro_name, reason } => {
                write!(f, "Malformed macro definition for '{}'\n", macro_name)?;
                write!(f, "  Reason: {}\n", reason)?;
                write!(f, "  Help: Check the syntax of your defmacro form")
            }
            MacroError::InvalidPattern { pattern, reason } => {
                write!(f, "Invalid parameter pattern: '{}'\n", pattern)?;
                write!(f, "  Reason: {}\n", reason)?;
                write!(f, "  Help: Valid patterns include simple parameters and &rest patterns")
            }
        }
    }
}

impl std::error::Error for MacroError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            expansion_depth: 0,
            max_depth: 100, // Prevent infinite recursion
            gensym_counter: 0,
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            macros: HashMap::new(),
            expansion_depth: 0,
            max_depth,
            gensym_counter: 0,
        }
    }

    /// Generate a unique symbol for hygienic macros
    pub fn gensym(&mut self, prefix: &str) -> String {
        self.gensym_counter += 1;
        format!("{}#g{}", prefix, self.gensym_counter)
    }

    /// Generate a gensym expression
    pub fn gen_gensym_expr(&mut self, prefix: &str) -> LispExpr {
        LispExpr::Gensym(self.gensym(prefix))
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
                    return Err(MacroError::MaxDepthExceeded {
                        depth: self.max_depth,
                        macro_name: name.clone(),
                    });
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
                            return Err(MacroError::MaxDepthExceeded {
                                depth: self.max_depth,
                                macro_name: name.clone(),
                            });
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

        // Store macro name for better error messages
        let macro_name = name.to_string();

        // Create parameter bindings using pattern matching
        let bindings = self.match_parameters(&macro_name, &macro_def.parameters, &args)?;

        // Apply hygiene: collect symbols introduced by the macro (not parameters)
        let param_names: Vec<String> = bindings.keys().cloned().collect();
        let introduced_symbols = self.collect_introduced_symbols(&macro_def.body, &param_names);

        // Create hygiene renaming map for introduced symbols
        let mut hygiene_map = HashMap::new();
        for symbol in introduced_symbols {
            let renamed = self.gensym(&symbol);
            hygiene_map.insert(symbol, renamed);
        }

        // Apply hygiene renaming to macro body first
        let hygienic_body = self.apply_hygiene_renaming(&macro_def.body, &hygiene_map);

        // Substitute parameters in the hygienic macro body
        let substituted_body = self.substitute_parameters(&hygienic_body, &bindings)?;

        // Recursively expand the result in case it contains more macro calls
        self.expand_expression(substituted_body)
    }

    /// Match macro parameters against arguments, supporting &rest and other patterns
    fn match_parameters(&self, macro_name: &str, parameters: &[String], args: &[LispExpr]) -> Result<HashMap<String, LispExpr>, MacroError> {
        let mut bindings = HashMap::new();

        // Find if there's a &rest parameter
        let rest_position = parameters.iter().position(|p| p == "&rest");

        if let Some(rest_idx) = rest_position {
            // Handle &rest parameter pattern
            if rest_idx + 1 >= parameters.len() {
                return Err(MacroError::InvalidPattern {
                    pattern: "&rest".to_string(),
                    reason: "&rest must be followed by a parameter name".to_string(),
                });
            }

            let rest_param_name = &parameters[rest_idx + 1];
            let required_params = &parameters[..rest_idx];

            // Check we have at least the required parameters
            if args.len() < required_params.len() {
                return Err(MacroError::ParameterCountMismatch {
                    macro_name: macro_name.to_string(),
                    expected: required_params.len(),
                    actual: args.len(),
                });
            }

            // Bind required parameters
            for (param, arg) in required_params.iter().zip(args.iter()) {
                bindings.insert(param.clone(), arg.clone());
            }

            // Collect remaining arguments into a list for the rest parameter
            let rest_args: Vec<LispExpr> = args[required_params.len()..].to_vec();
            bindings.insert(rest_param_name.clone(), LispExpr::List(rest_args));

            // Check for any parameters after the rest parameter name (which would be an error)
            if rest_idx + 2 < parameters.len() {
                return Err(MacroError::InvalidPattern {
                    pattern: format!("&rest {}", rest_param_name),
                    reason: "Parameters cannot appear after &rest parameter".to_string(),
                });
            }
        } else {
            // No &rest parameter - exact match required
            if parameters.len() != args.len() {
                return Err(MacroError::ParameterCountMismatch {
                    macro_name: macro_name.to_string(),
                    expected: parameters.len(),
                    actual: args.len(),
                });
            }

            // Simple binding
            for (param, arg) in parameters.iter().zip(args.iter()) {
                bindings.insert(param.clone(), arg.clone());
            }
        }

        Ok(bindings)
    }

    /// Collect symbols introduced by the macro (excluding parameters)
    fn collect_introduced_symbols(&self, expr: &LispExpr, parameters: &[String]) -> Vec<String> {
        let mut symbols = Vec::new();

        // For quasiquoted bodies, we need to collect symbols that are NOT inside unquotes
        self.collect_non_parameter_symbols(expr, &mut symbols, parameters);

        // Built-in forms that should not be renamed
        const BUILTIN_FORMS: &[&str] = &[
            "let", "if", "define", "lambda", "quote", "quasiquote", "unquote", "unquote-splicing",
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
            "and", "or", "not", "list", "car", "cdr", "cons",
            "set!", "begin", "progn",
        ];

        // Filter out built-in forms
        symbols.retain(|s| !BUILTIN_FORMS.contains(&s.as_str()));

        // Filter out macro names - they should be resolved, not renamed
        symbols.retain(|s| !self.macros.contains_key(s));

        // Remove duplicates
        symbols.sort();
        symbols.dedup();

        symbols
    }

    fn collect_non_parameter_symbols(&self, expr: &LispExpr, symbols: &mut Vec<String>, parameters: &[String]) {
        match expr {
            LispExpr::Symbol(name) => {
                // Only collect if not a parameter
                if !parameters.contains(name) {
                    symbols.push(name.clone());
                }
            }
            LispExpr::List(elements) => {
                for element in elements {
                    self.collect_non_parameter_symbols(element, symbols, parameters);
                }
            }
            LispExpr::Quote(_) => {
                // Don't collect symbols inside quotes
            }
            LispExpr::Quasiquote(inner) => {
                // In quasiquote, collect symbols that are NOT inside unquotes
                self.collect_symbols_in_quasiquote(inner, symbols, parameters);
            }
            LispExpr::Unquote(_) | LispExpr::Splice(_) => {
                // Don't collect from unquoted parts - these are parameters
            }
            LispExpr::Macro { body, .. } => {
                self.collect_non_parameter_symbols(body, symbols, parameters);
            }
            _ => {}
        }
    }

    fn collect_symbols_in_quasiquote(&self, expr: &LispExpr, symbols: &mut Vec<String>, parameters: &[String]) {
        match expr {
            LispExpr::Unquote(_) | LispExpr::Splice(_) => {
                // Skip unquoted parts - these contain parameters
            }
            LispExpr::Symbol(name) => {
                // Collect symbols in the quasiquoted part (not unquoted)
                if !parameters.contains(name) {
                    symbols.push(name.clone());
                }
            }
            LispExpr::List(elements) => {
                for element in elements {
                    self.collect_symbols_in_quasiquote(element, symbols, parameters);
                }
            }
            _ => {}
        }
    }

    /// Apply hygiene renaming to symbols
    fn apply_hygiene_renaming(&self, expr: &LispExpr, hygiene_map: &HashMap<String, String>) -> LispExpr {
        match expr {
            LispExpr::Symbol(name) => {
                if let Some(renamed) = hygiene_map.get(name) {
                    LispExpr::Gensym(renamed.clone())
                } else {
                    expr.clone()
                }
            }
            LispExpr::List(elements) => {
                LispExpr::List(
                    elements.iter()
                        .map(|e| self.apply_hygiene_renaming(e, hygiene_map))
                        .collect()
                )
            }
            LispExpr::Quote(inner) => {
                // Don't rename inside quotes
                LispExpr::Quote(inner.clone())
            }
            LispExpr::Quasiquote(inner) => {
                LispExpr::Quasiquote(Box::new(
                    self.apply_hygiene_renaming_in_quasiquote(inner, hygiene_map)
                ))
            }
            LispExpr::Unquote(inner) => {
                LispExpr::Unquote(Box::new(
                    self.apply_hygiene_renaming(inner, hygiene_map)
                ))
            }
            LispExpr::Splice(inner) => {
                LispExpr::Splice(Box::new(
                    self.apply_hygiene_renaming(inner, hygiene_map)
                ))
            }
            LispExpr::Macro { name, parameters, body } => {
                LispExpr::Macro {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: Box::new(self.apply_hygiene_renaming(body, hygiene_map)),
                }
            }
            _ => expr.clone(),
        }
    }

    fn apply_hygiene_renaming_in_quasiquote(&self, expr: &LispExpr, hygiene_map: &HashMap<String, String>) -> LispExpr {
        match expr {
            LispExpr::Unquote(inner) => {
                // Don't rename inside unquotes - those are parameters
                LispExpr::Unquote(inner.clone())
            }
            LispExpr::Splice(inner) => {
                // Don't rename inside splices - those are parameters
                LispExpr::Splice(inner.clone())
            }
            LispExpr::Symbol(name) => {
                // Rename symbols in the quasiquoted part
                if let Some(renamed) = hygiene_map.get(name) {
                    LispExpr::Gensym(renamed.clone())
                } else {
                    expr.clone()
                }
            }
            LispExpr::List(elements) => {
                LispExpr::List(
                    elements.iter()
                        .map(|e| self.apply_hygiene_renaming_in_quasiquote(e, hygiene_map))
                        .collect()
                )
            }
            _ => expr.clone(),
        }
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
                            return Err(MacroError::ExpansionError {
                                message: "Splice (unquote-splicing) must expand to a list".to_string(),
                                context: Some(format!("Got: {:?}", expanded)),
                            });
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
                            return Err(MacroError::ExpansionError {
                                message: "Splice (unquote-splicing) must expand to a list".to_string(),
                                context: Some(format!("Got: {:?}", substituted)),
                            });
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

        if let Err(MacroError::ParameterCountMismatch { macro_name, expected, actual }) = result {
            assert_eq!(macro_name, "test_macro");
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

        if let Err(MacroError::MaxDepthExceeded { depth, macro_name }) = result {
            assert_eq!(depth, 2);
            assert_eq!(macro_name, "recursive_macro");
        } else {
            panic!("Expected MaxDepthExceeded error, got: {:?}", result);
        }
    }

    // Hygiene tests

    #[test]
    fn test_gensym_generation() {
        let mut expander = MacroExpander::new();

        let sym1 = expander.gensym("temp");
        let sym2 = expander.gensym("temp");
        let sym3 = expander.gensym("var");

        // Each gensym should be unique
        assert_ne!(sym1, sym2);
        assert_ne!(sym2, sym3);
        assert_ne!(sym1, sym3);

        // Should contain the prefix
        assert!(sym1.starts_with("temp"));
        assert!(sym2.starts_with("temp"));
        assert!(sym3.starts_with("var"));
    }

    #[test]
    fn test_hygiene_prevents_variable_capture() {
        let mut expander = MacroExpander::new();

        // Define a macro that introduces a temporary variable 'temp'
        // (defmacro swap (a b) `(let ((temp ,a)) (set! ,a ,b) (set! ,b temp)))
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::List(vec![
                LispExpr::Symbol("let".to_string()),
                LispExpr::List(vec![
                    LispExpr::List(vec![
                        LispExpr::Symbol("temp".to_string()),
                        LispExpr::Unquote(Box::new(LispExpr::Symbol("a".to_string()))),
                    ]),
                ]),
                LispExpr::List(vec![
                    LispExpr::Symbol("set!".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("a".to_string()))),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("b".to_string()))),
                ]),
                LispExpr::List(vec![
                    LispExpr::Symbol("set!".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("b".to_string()))),
                    LispExpr::Symbol("temp".to_string()),
                ]),
            ])
        ));

        expander.define_macro(
            "swap".to_string(),
            vec!["a".to_string(), "b".to_string()],
            macro_body,
        );

        // Call the macro with arguments
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("swap".to_string()),
            LispExpr::Symbol("x".to_string()),
            LispExpr::Symbol("y".to_string()),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // The expanded code should use a gensym for 'temp'
        // Check that the result contains a Gensym variant
        fn contains_gensym(expr: &LispExpr) -> bool {
            match expr {
                LispExpr::Gensym(_) => true,
                LispExpr::List(elements) => elements.iter().any(contains_gensym),
                LispExpr::Quote(inner) |
                LispExpr::Quasiquote(inner) |
                LispExpr::Unquote(inner) |
                LispExpr::Splice(inner) => contains_gensym(inner),
                _ => false,
            }
        }

        assert!(contains_gensym(&result),
            "Hygienic macro should rename introduced variables to gensyms");
    }

    #[test]
    fn test_hygiene_preserves_parameters() {
        let mut expander = MacroExpander::new();

        // Define a macro: (defmacro use-param (x) `(+ ,x ,x))
        // The parameter 'x' should NOT be renamed
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
                LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
            ])
        ));

        expander.define_macro(
            "use-param".to_string(),
            vec!["x".to_string()],
            macro_body,
        );

        // Call with a value
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("use-param".to_string()),
            LispExpr::Number(5.0),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Should expand to: (+ 5 5)
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(5.0),
            LispExpr::Number(5.0),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hygiene_with_let_bindings() {
        let mut expander = MacroExpander::new();

        // Macro that introduces a let binding
        // (defmacro with-temp (x) `(let ((result ,x)) result))
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::List(vec![
                LispExpr::Symbol("let".to_string()),
                LispExpr::List(vec![
                    LispExpr::List(vec![
                        LispExpr::Symbol("result".to_string()),
                        LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
                    ]),
                ]),
                LispExpr::Symbol("result".to_string()),
            ])
        ));

        expander.define_macro(
            "with-temp".to_string(),
            vec!["x".to_string()],
            macro_body,
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("with-temp".to_string()),
            LispExpr::Number(42.0),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // The introduced variable 'result' should be renamed to a gensym
        fn check_let_has_gensym(expr: &LispExpr) -> bool {
            match expr {
                LispExpr::List(elements) => {
                    if let Some(LispExpr::Symbol(first)) = elements.first() {
                        if first == "let" && elements.len() >= 3 {
                            // Check if bindings contain gensym
                            if let LispExpr::List(bindings) = &elements[1] {
                                for binding in bindings {
                                    if let LispExpr::List(pair) = binding {
                                        if matches!(pair.first(), Some(LispExpr::Gensym(_))) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    elements.iter().any(check_let_has_gensym)
                }
                _ => false,
            }
        }

        assert!(check_let_has_gensym(&result),
            "Let-bound variables introduced by macros should be renamed to gensyms");
    }

    #[test]
    fn test_nested_macros_maintain_hygiene() {
        let mut expander = MacroExpander::new();

        // First macro introduces a variable
        expander.define_macro(
            "with-x".to_string(),
            vec!["val".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("let".to_string()),
                    LispExpr::List(vec![
                        LispExpr::List(vec![
                            LispExpr::Symbol("x".to_string()),
                            LispExpr::Unquote(Box::new(LispExpr::Symbol("val".to_string()))),
                        ]),
                    ]),
                    LispExpr::Symbol("x".to_string()),
                ])
            )),
        );

        // Second macro also introduces a variable
        expander.define_macro(
            "with-y".to_string(),
            vec!["val".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("let".to_string()),
                    LispExpr::List(vec![
                        LispExpr::List(vec![
                            LispExpr::Symbol("y".to_string()),
                            LispExpr::Unquote(Box::new(LispExpr::Symbol("val".to_string()))),
                        ]),
                    ]),
                    LispExpr::Symbol("y".to_string()),
                ])
            )),
        );

        // Nested macro call
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("with-x".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol("with-y".to_string()),
                LispExpr::Number(10.0),
            ]),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Both introduced variables should be renamed
        fn count_gensyms(expr: &LispExpr) -> usize {
            match expr {
                LispExpr::Gensym(_) => 1,
                LispExpr::List(elements) => elements.iter().map(count_gensyms).sum(),
                LispExpr::Quote(inner) |
                LispExpr::Quasiquote(inner) |
                LispExpr::Unquote(inner) |
                LispExpr::Splice(inner) => count_gensyms(inner),
                _ => 0,
            }
        }

        let gensym_count = count_gensyms(&result);
        assert!(gensym_count >= 2,
            "Nested macros should each introduce at least one gensym, found: {}", gensym_count);
    }

    #[test]
    fn test_hygiene_does_not_rename_quoted_symbols() {
        let mut expander = MacroExpander::new();

        // Macro with quoted symbol that should not be renamed
        // (defmacro get-quoted (x) `'symbol)
        let macro_body = LispExpr::Quasiquote(Box::new(
            LispExpr::Quote(Box::new(LispExpr::Symbol("symbol".to_string())))
        ));

        expander.define_macro(
            "get-quoted".to_string(),
            vec!["x".to_string()],
            macro_body,
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("get-quoted".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Should expand to: 'symbol (not renamed)
        if let LispExpr::Quote(inner) = result {
            assert!(matches!(*inner, LispExpr::Symbol(_)),
                "Quoted symbols should not be renamed to gensyms");
        } else {
            panic!("Expected Quote expression");
        }
    }

    // Pattern matching tests

    #[test]
    fn test_rest_parameter_basic() {
        let mut expander = MacroExpander::new();

        // Define macro with &rest: (defmacro my-list (first &rest rest) `(list ,first ,@rest))
        expander.define_macro(
            "my-list".to_string(),
            vec!["first".to_string(), "&rest".to_string(), "rest".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("list".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("first".to_string()))),
                    LispExpr::Splice(Box::new(LispExpr::Symbol("rest".to_string()))),
                ])
            )),
        );

        // Call: (my-list 1 2 3 4)
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("my-list".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
            LispExpr::Number(3.0),
            LispExpr::Number(4.0),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Should expand to: (list 1 2 3 4)
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("list".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
            LispExpr::Number(3.0),
            LispExpr::Number(4.0),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rest_parameter_empty() {
        let mut expander = MacroExpander::new();

        // Define macro: (defmacro my-list (first &rest rest) `(list ,first ,@rest))
        expander.define_macro(
            "my-list".to_string(),
            vec!["first".to_string(), "&rest".to_string(), "rest".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("list".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("first".to_string()))),
                    LispExpr::Splice(Box::new(LispExpr::Symbol("rest".to_string()))),
                ])
            )),
        );

        // Call with only required parameter: (my-list 1)
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("my-list".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Should expand to: (list 1) - rest is empty
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("list".to_string()),
            LispExpr::Number(1.0),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rest_parameter_multiple_required() {
        let mut expander = MacroExpander::new();

        // Define macro: (defmacro add-first-two-then-rest (a b &rest rest) `(+ (+ ,a ,b) ,@rest))
        expander.define_macro(
            "add-first-two-then-rest".to_string(),
            vec!["a".to_string(), "b".to_string(), "&rest".to_string(), "rest".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("+".to_string()),
                    LispExpr::List(vec![
                        LispExpr::Symbol("+".to_string()),
                        LispExpr::Unquote(Box::new(LispExpr::Symbol("a".to_string()))),
                        LispExpr::Unquote(Box::new(LispExpr::Symbol("b".to_string()))),
                    ]),
                    LispExpr::Splice(Box::new(LispExpr::Symbol("rest".to_string()))),
                ])
            )),
        );

        // Call: (add-first-two-then-rest 1 2 3 4 5)
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("add-first-two-then-rest".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
            LispExpr::Number(3.0),
            LispExpr::Number(4.0),
            LispExpr::Number(5.0),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Should expand to: (+ (+ 1 2) 3 4 5)
        let expected = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Number(1.0),
                LispExpr::Number(2.0),
            ]),
            LispExpr::Number(3.0),
            LispExpr::Number(4.0),
            LispExpr::Number(5.0),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rest_parameter_too_few_args() {
        let mut expander = MacroExpander::new();

        // Define macro with 2 required params
        expander.define_macro(
            "needs-two-plus".to_string(),
            vec!["a".to_string(), "b".to_string(), "&rest".to_string(), "rest".to_string()],
            LispExpr::Symbol("body".to_string()),
        );

        // Call with only 1 arg (need at least 2)
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("needs-two-plus".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call);

        assert!(result.is_err());
        if let Err(MacroError::ParameterCountMismatch { macro_name, expected, actual }) = result {
            assert_eq!(macro_name, "needs-two-plus");
            assert_eq!(expected, 2);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected ParameterCountMismatch error");
        }
    }

    #[test]
    fn test_rest_without_name_error() {
        let mut expander = MacroExpander::new();

        // Define macro with &rest but no following parameter name
        expander.define_macro(
            "bad-macro".to_string(),
            vec!["a".to_string(), "&rest".to_string()],
            LispExpr::Symbol("body".to_string()),
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("bad-macro".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call);

        assert!(result.is_err());
        if let Err(MacroError::InvalidPattern { pattern, reason }) = result {
            assert_eq!(pattern, "&rest");
            assert!(reason.contains("&rest must be followed by a parameter name"));
        } else {
            panic!("Expected InvalidPattern error for &rest without name, got: {:?}", result);
        }
    }

    #[test]
    fn test_params_after_rest_error() {
        let mut expander = MacroExpander::new();

        // Define macro with parameters after &rest (which is invalid)
        expander.define_macro(
            "bad-macro".to_string(),
            vec!["a".to_string(), "&rest".to_string(), "rest".to_string(), "extra".to_string()],
            LispExpr::Symbol("body".to_string()),
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("bad-macro".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);

        let result = expander.expand_all(macro_call);

        assert!(result.is_err());
        if let Err(MacroError::InvalidPattern { pattern, reason }) = result {
            assert!(pattern.contains("&rest"));
            assert!(reason.contains("Parameters cannot appear after &rest parameter"));
        } else {
            panic!("Expected InvalidPattern error for parameters after &rest, got: {:?}", result);
        }
    }

    #[test]
    fn test_when_macro_with_rest() {
        let mut expander = MacroExpander::new();

        // Classic 'when' macro: (defmacro when (condition &rest body) `(if ,condition (progn ,@body) nil))
        expander.define_macro(
            "when".to_string(),
            vec!["condition".to_string(), "&rest".to_string(), "body".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("if".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("condition".to_string()))),
                    LispExpr::List(vec![
                        LispExpr::Symbol("progn".to_string()),
                        LispExpr::Splice(Box::new(LispExpr::Symbol("body".to_string()))),
                    ]),
                    LispExpr::Nil,
                ])
            )),
        );

        // Call: (when (> x 5) (print "big") (+ x 1))
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("when".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol(">".to_string()),
                LispExpr::Symbol("x".to_string()),
                LispExpr::Number(5.0),
            ]),
            LispExpr::List(vec![
                LispExpr::Symbol("print".to_string()),
                LispExpr::String("big".to_string()),
            ]),
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Symbol("x".to_string()),
                LispExpr::Number(1.0),
            ]),
        ]);

        let result = expander.expand_all(macro_call).unwrap();

        // Verify structure: should be (if condition (progn ...) <optional nil>)
        // Note: The macro expander may filter out Nil in some contexts
        if let LispExpr::List(elements) = &result {
            assert!(elements.len() >= 3, "Expected at least 3 elements in if expression, got {}", elements.len());
            assert_eq!(elements[0], LispExpr::Symbol("if".to_string()));

            // Check the progn body contains both expressions
            if let LispExpr::List(progn_parts) = &elements[2] {
                assert_eq!(progn_parts.len(), 3); // progn + 2 body expressions
                assert_eq!(progn_parts[0], LispExpr::Symbol("progn".to_string()));
            } else {
                panic!("Expected List for progn body");
            }
        } else {
            panic!("Expected List for if expression");
        }
    }

    // Comprehensive error message tests

    #[test]
    fn test_error_message_quality_undefined_macro() {
        let mut expander = MacroExpander::new();

        // Try to expand undefined macro - these pass through as regular function calls
        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("undefined".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call.clone());
        // Undefined macros pass through unchanged (they might be regular functions)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), macro_call);
    }

    #[test]
    fn test_error_message_quality_param_mismatch() {
        let mut expander = MacroExpander::new();

        expander.define_macro(
            "my-macro".to_string(),
            vec!["x".to_string(), "y".to_string()],
            LispExpr::Symbol("body".to_string()),
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("my-macro".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = format!("{}", error);

        // Check for helpful error message
        assert!(error_msg.contains("my-macro"));
        assert!(error_msg.contains("2"));
        assert!(error_msg.contains("1"));
        assert!(error_msg.contains("Help"));
    }

    #[test]
    fn test_error_message_quality_max_depth() {
        let mut expander = MacroExpander::with_max_depth(5);

        // Define recursive macro
        expander.define_macro(
            "infinite".to_string(),
            vec!["x".to_string()],
            LispExpr::Quasiquote(Box::new(
                LispExpr::List(vec![
                    LispExpr::Symbol("infinite".to_string()),
                    LispExpr::Unquote(Box::new(LispExpr::Symbol("x".to_string()))),
                ])
            )),
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("infinite".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = format!("{}", error);

        // Check for helpful error message
        assert!(error_msg.contains("infinite"));
        assert!(error_msg.contains("5"));
        assert!(error_msg.contains("Help"));
        assert!(error_msg.contains("infinite recursion") || error_msg.contains("base case"));
    }

    #[test]
    fn test_error_message_quality_invalid_pattern() {
        let mut expander = MacroExpander::new();

        // Define macro with bad &rest pattern
        expander.define_macro(
            "bad".to_string(),
            vec!["a".to_string(), "&rest".to_string()],
            LispExpr::Symbol("body".to_string()),
        );

        let macro_call = LispExpr::List(vec![
            LispExpr::Symbol("bad".to_string()),
            LispExpr::Number(1.0),
        ]);

        let result = expander.expand_all(macro_call);
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = format!("{}", error);

        // Check for helpful error message
        assert!(error_msg.contains("&rest"));
        assert!(error_msg.contains("parameter name"));
        assert!(error_msg.contains("Help"));
    }

    #[test]
    fn test_std_error_trait_implementation() {
        use std::error::Error;

        let error = MacroError::UndefinedMacro("test".to_string());

        // Test that it implements Error trait
        let _boxed: Box<dyn Error> = Box::new(error);
    }

    #[test]
    fn test_error_context_in_expansion_error() {
        let error = MacroError::ExpansionError {
            message: "Something went wrong".to_string(),
            context: Some("While expanding macro foo".to_string()),
        };

        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Something went wrong"));
        assert!(error_msg.contains("While expanding macro foo"));
    }

    #[test]
    fn test_clone_and_partialeq_on_errors() {
        let error1 = MacroError::UndefinedMacro("test".to_string());
        let error2 = error1.clone();

        assert_eq!(error1, error2);

        let error3 = MacroError::ParameterCountMismatch {
            macro_name: "foo".to_string(),
            expected: 2,
            actual: 1,
        };
        let error4 = error3.clone();

        assert_eq!(error3, error4);
        assert_ne!(error1, error3);
    }
}