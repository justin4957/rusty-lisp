use crate::ast::LispExpr;
use std::collections::{HashSet, HashMap};
use std::fmt;

/// Validation rules for AST safety checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRule {
    /// Basic type safety checks
    TypeSafety,
    /// Resource bounds (infinite loops/recursion)
    ResourceBounds,
    /// FFI and unsafe Rust restrictions
    FFIRestrictions,
    /// Computational complexity limits
    ComplexityLimits,
}

/// Type information for basic type inference
#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    Number,
    String,
    Bool,
    List(Box<InferredType>),
    Symbol,
    Unknown,
    Any,
}

/// Validation errors with context
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub rule: ValidationRule,
    pub message: String,
    pub context: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} violation: {}", self.rule, self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, "\n  Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

pub type ValidationResult = Result<(), ValidationError>;

/// Main validator trait for composable validation
pub trait ASTValidator {
    fn validate(&self, expr: &LispExpr) -> ValidationResult;
    fn enabled_rules(&self) -> Vec<ValidationRule>;
}

/// Composite validator that runs multiple validation rules
pub struct CompositeValidator {
    validators: Vec<Box<dyn ASTValidator>>,
}

impl CompositeValidator {
    pub fn new() -> Self {
        CompositeValidator {
            validators: Vec::new(),
        }
    }

    pub fn add_validator(mut self, validator: Box<dyn ASTValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn validate_all(&self, expr: &LispExpr) -> Result<(), Vec<ValidationError>> {
        let errors: Vec<ValidationError> = self
            .validators
            .iter()
            .filter_map(|v| v.validate(expr).err())
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Type safety validator
pub struct TypeSafetyValidator {
    type_environment: HashMap<String, InferredType>,
}

impl TypeSafetyValidator {
    pub fn new() -> Self {
        TypeSafetyValidator {
            type_environment: HashMap::new(),
        }
    }

    fn infer_type(&self, expr: &LispExpr) -> InferredType {
        match expr {
            LispExpr::Number(_) => InferredType::Number,
            LispExpr::String(_) => InferredType::String,
            LispExpr::Bool(_) => InferredType::Bool,
            LispExpr::Nil => InferredType::Symbol,
            LispExpr::Symbol(s) => {
                self.type_environment
                    .get(s)
                    .cloned()
                    .unwrap_or(InferredType::Unknown)
            }
            LispExpr::List(elements) => {
                if elements.is_empty() {
                    InferredType::List(Box::new(InferredType::Any))
                } else if let Some(op) = elements[0].as_symbol() {
                    // Infer return type based on operation
                    match op {
                        "+" | "-" | "*" | "/" => InferredType::Number,
                        "<" | ">" | "<=" | ">=" | "=" => InferredType::Bool,
                        "if" => {
                            // if expressions return the type of their branches
                            if elements.len() >= 3 {
                                self.infer_type(&elements[2])
                            } else {
                                InferredType::Unknown
                            }
                        }
                        _ => InferredType::Unknown,
                    }
                } else {
                    let first_type = self.infer_type(&elements[0]);
                    InferredType::List(Box::new(first_type))
                }
            }
            LispExpr::Quote(_) | LispExpr::Quasiquote(_) => InferredType::Any,
            _ => InferredType::Unknown,
        }
    }

    fn validate_operation(&self, op: &str, args: &[LispExpr]) -> ValidationResult {
        match op {
            "+" | "-" | "*" | "/" => {
                // Arithmetic operations require numeric operands
                for arg in args {
                    let arg_type = self.infer_type(arg);
                    if !matches!(arg_type, InferredType::Number | InferredType::Unknown) {
                        return Err(ValidationError {
                            rule: ValidationRule::TypeSafety,
                            message: format!(
                                "Type mismatch: arithmetic operation '{}' requires numeric operands, got {:?}",
                                op, arg_type
                            ),
                            context: Some(format!("{:?}", arg)),
                        });
                    }
                }
                Ok(())
            }
            "<" | ">" | "<=" | ">=" | "=" => {
                // Comparison operations require compatible types
                if args.len() == 2 {
                    let left_type = self.infer_type(&args[0]);
                    let right_type = self.infer_type(&args[1]);

                    if !self.types_compatible(&left_type, &right_type) {
                        return Err(ValidationError {
                            rule: ValidationRule::TypeSafety,
                            message: format!(
                                "Type mismatch: comparison '{}' requires compatible types, got {:?} and {:?}",
                                op, left_type, right_type
                            ),
                            context: Some(format!("{:?} vs {:?}", args[0], args[1])),
                        });
                    }
                }
                Ok(())
            }
            _ => Ok(()), // Unknown operations pass through
        }
    }

    fn types_compatible(&self, t1: &InferredType, t2: &InferredType) -> bool {
        matches!(
            (t1, t2),
            (InferredType::Unknown, _)
                | (_, InferredType::Unknown)
                | (InferredType::Any, _)
                | (_, InferredType::Any)
        ) || t1 == t2
    }
}

impl ASTValidator for TypeSafetyValidator {
    fn validate(&self, expr: &LispExpr) -> ValidationResult {
        match expr {
            LispExpr::List(elements) if !elements.is_empty() => {
                // Check if this is an operation
                if let Some(op) = elements[0].as_symbol() {
                    self.validate_operation(op, &elements[1..])?;
                }

                // Recursively validate all elements
                for elem in elements {
                    self.validate(elem)?;
                }
                Ok(())
            }
            LispExpr::List(elements) => {
                // Empty list or nested lists
                for elem in elements {
                    self.validate(elem)?;
                }
                Ok(())
            }
            LispExpr::Quote(inner)
            | LispExpr::Quasiquote(inner)
            | LispExpr::Unquote(inner)
            | LispExpr::Splice(inner) => self.validate(inner),
            LispExpr::Macro { body, .. } => self.validate(body),
            LispExpr::MacroCall { args, .. } => {
                for arg in args {
                    self.validate(arg)?;
                }
                Ok(())
            }
            _ => Ok(()), // Atoms are always valid
        }
    }

    fn enabled_rules(&self) -> Vec<ValidationRule> {
        vec![ValidationRule::TypeSafety]
    }
}

/// Resource bounds validator (detects infinite loops/recursion)
pub struct ResourceBoundsValidator {
    max_recursion_depth: usize,
}

impl ResourceBoundsValidator {
    pub fn new() -> Self {
        ResourceBoundsValidator {
            max_recursion_depth: 100,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    fn check_immediate_recursion(&self, expr: &LispExpr, context: &HashSet<String>) -> ValidationResult {
        match expr {
            LispExpr::List(elements) if !elements.is_empty() => {
                // Check for (define (foo) (foo)) pattern
                if let Some("define") = elements[0].as_symbol() {
                    if elements.len() >= 3 {
                        // Extract function name
                        let fn_name = if let Some(LispExpr::List(def_list)) = elements.get(1) {
                            def_list.get(0).and_then(|e| e.as_symbol())
                        } else {
                            elements.get(1).and_then(|e| e.as_symbol())
                        };

                        if let Some(name) = fn_name {
                            // Check if body immediately calls itself without any base case
                            let body = &elements[2];
                            if self.is_immediate_self_call(body, name) {
                                return Err(ValidationError {
                                    rule: ValidationRule::ResourceBounds,
                                    message: format!(
                                        "Infinite recursion detected: function '{}' calls itself without any conditional base case",
                                        name
                                    ),
                                    context: Some(format!("{:?}", expr)),
                                });
                            }
                        }
                    }
                }

                // Recursively check nested expressions
                for elem in elements {
                    self.check_immediate_recursion(elem, context)?;
                }
                Ok(())
            }
            LispExpr::Quote(inner)
            | LispExpr::Quasiquote(inner)
            | LispExpr::Unquote(inner)
            | LispExpr::Splice(inner) => self.check_immediate_recursion(inner, context),
            LispExpr::Macro { body, .. } => self.check_immediate_recursion(body, context),
            LispExpr::MacroCall { args, .. } => {
                for arg in args {
                    self.check_immediate_recursion(arg, context)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn is_immediate_self_call(&self, expr: &LispExpr, fn_name: &str) -> bool {
        match expr {
            LispExpr::List(elements) if !elements.is_empty() => {
                // Check if this is a direct call to the function
                if let Some(call_name) = elements[0].as_symbol() {
                    if call_name == fn_name {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl ASTValidator for ResourceBoundsValidator {
    fn validate(&self, expr: &LispExpr) -> ValidationResult {
        let context = HashSet::new();
        self.check_immediate_recursion(expr, &context)
    }

    fn enabled_rules(&self) -> Vec<ValidationRule> {
        vec![ValidationRule::ResourceBounds]
    }
}

/// FFI restrictions validator
pub struct FFIRestrictionsValidator {
    allowed_ffi_functions: HashSet<String>,
}

impl FFIRestrictionsValidator {
    pub fn new() -> Self {
        FFIRestrictionsValidator {
            allowed_ffi_functions: HashSet::new(),
        }
    }

    pub fn allow_function(mut self, name: String) -> Self {
        self.allowed_ffi_functions.insert(name);
        self
    }

    fn check_unsafe_operations(&self, expr: &LispExpr) -> ValidationResult {
        match expr {
            LispExpr::List(elements) if !elements.is_empty() => {
                // Check for unsafe Rust operations
                if let Some(op) = elements[0].as_symbol() {
                    if op.starts_with("rust-unsafe") || op.starts_with("ffi-") {
                        if !self.allowed_ffi_functions.contains(op) {
                            return Err(ValidationError {
                                rule: ValidationRule::FFIRestrictions,
                                message: format!(
                                    "FFI restriction: unsafe operation '{}' is not allowed",
                                    op
                                ),
                                context: Some(format!("{:?}", expr)),
                            });
                        }
                    }
                }

                // Recursively check nested expressions
                for elem in elements {
                    self.check_unsafe_operations(elem)?;
                }
                Ok(())
            }
            LispExpr::Quote(inner)
            | LispExpr::Quasiquote(inner)
            | LispExpr::Unquote(inner)
            | LispExpr::Splice(inner) => self.check_unsafe_operations(inner),
            LispExpr::Macro { body, .. } => self.check_unsafe_operations(body),
            LispExpr::MacroCall { args, .. } => {
                for arg in args {
                    self.check_unsafe_operations(arg)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl ASTValidator for FFIRestrictionsValidator {
    fn validate(&self, expr: &LispExpr) -> ValidationResult {
        self.check_unsafe_operations(expr)
    }

    fn enabled_rules(&self) -> Vec<ValidationRule> {
        vec![ValidationRule::FFIRestrictions]
    }
}

/// Complexity limits validator
pub struct ComplexityLimitsValidator {
    max_nesting_depth: usize,
}

impl ComplexityLimitsValidator {
    pub fn new() -> Self {
        ComplexityLimitsValidator {
            max_nesting_depth: 50,
        }
    }

    pub fn with_max_nesting(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    fn check_nesting_depth(&self, expr: &LispExpr, current_depth: usize) -> ValidationResult {
        if current_depth > self.max_nesting_depth {
            return Err(ValidationError {
                rule: ValidationRule::ComplexityLimits,
                message: format!(
                    "Complexity limit exceeded: nesting depth {} exceeds maximum {}",
                    current_depth, self.max_nesting_depth
                ),
                context: None,
            });
        }

        match expr {
            LispExpr::List(elements) => {
                for elem in elements {
                    self.check_nesting_depth(elem, current_depth + 1)?;
                }
                Ok(())
            }
            LispExpr::Quote(inner)
            | LispExpr::Quasiquote(inner)
            | LispExpr::Unquote(inner)
            | LispExpr::Splice(inner) => self.check_nesting_depth(inner, current_depth + 1),
            LispExpr::Macro { body, .. } => self.check_nesting_depth(body, current_depth + 1),
            LispExpr::MacroCall { args, .. } => {
                for arg in args {
                    self.check_nesting_depth(arg, current_depth + 1)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl ASTValidator for ComplexityLimitsValidator {
    fn validate(&self, expr: &LispExpr) -> ValidationResult {
        self.check_nesting_depth(expr, 0)
    }

    fn enabled_rules(&self) -> Vec<ValidationRule> {
        vec![ValidationRule::ComplexityLimits]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_safety_arithmetic_with_numbers() {
        let validator = TypeSafetyValidator::new();
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);
        assert!(validator.validate(&expr).is_ok());
    }

    #[test]
    fn test_type_safety_arithmetic_with_string_fails() {
        let validator = TypeSafetyValidator::new();
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::String("hello".to_string()),
            LispExpr::Number(42.0),
        ]);
        let result = validator.validate(&expr);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.rule, ValidationRule::TypeSafety);
            assert!(e.message.contains("arithmetic operation"));
        }
    }

    #[test]
    fn test_type_safety_nested_arithmetic() {
        let validator = TypeSafetyValidator::new();
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::Number(2.0),
                LispExpr::Number(3.0),
            ]),
        ]);
        assert!(validator.validate(&expr).is_ok());
    }

    #[test]
    fn test_resource_bounds_immediate_recursion() {
        let validator = ResourceBoundsValidator::new();
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("define".to_string()),
            LispExpr::List(vec![LispExpr::Symbol("infinite-loop".to_string())]),
            LispExpr::List(vec![LispExpr::Symbol("infinite-loop".to_string())]),
        ]);
        let result = validator.validate(&expr);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.rule, ValidationRule::ResourceBounds);
            assert!(e.message.contains("Infinite recursion"));
        }
    }

    #[test]
    fn test_resource_bounds_valid_recursion_with_condition() {
        let validator = ResourceBoundsValidator::new();
        // (define (countdown n) (if (= n 0) 0 (countdown (- n 1))))
        // This has a conditional, so it should pass
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("define".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol("countdown".to_string()),
                LispExpr::Symbol("n".to_string()),
            ]),
            LispExpr::List(vec![
                LispExpr::Symbol("if".to_string()),
                LispExpr::List(vec![
                    LispExpr::Symbol("=".to_string()),
                    LispExpr::Symbol("n".to_string()),
                    LispExpr::Number(0.0),
                ]),
                LispExpr::Number(0.0),
                LispExpr::List(vec![
                    LispExpr::Symbol("countdown".to_string()),
                    LispExpr::List(vec![
                        LispExpr::Symbol("-".to_string()),
                        LispExpr::Symbol("n".to_string()),
                        LispExpr::Number(1.0),
                    ]),
                ]),
            ]),
        ]);
        // This should pass because the body is an if expression, not a direct call
        assert!(validator.validate(&expr).is_ok());
    }

    #[test]
    fn test_ffi_restrictions_unsafe_operation() {
        let validator = FFIRestrictionsValidator::new();
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("rust-unsafe".to_string()),
            LispExpr::String("std::ptr::null()".to_string()),
        ]);
        let result = validator.validate(&expr);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.rule, ValidationRule::FFIRestrictions);
            assert!(e.message.contains("unsafe operation"));
        }
    }

    #[test]
    fn test_ffi_restrictions_allowed_operation() {
        let validator = FFIRestrictionsValidator::new()
            .allow_function("rust-unsafe".to_string());
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("rust-unsafe".to_string()),
            LispExpr::String("std::ptr::null()".to_string()),
        ]);
        assert!(validator.validate(&expr).is_ok());
    }

    #[test]
    fn test_complexity_limits_shallow_nesting() {
        let validator = ComplexityLimitsValidator::new();
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);
        assert!(validator.validate(&expr).is_ok());
    }

    #[test]
    fn test_complexity_limits_deep_nesting_fails() {
        let validator = ComplexityLimitsValidator::new().with_max_nesting(5);

        // Create deeply nested expression
        let mut expr = LispExpr::Number(1.0);
        for _ in 0..10 {
            expr = LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                expr,
                LispExpr::Number(1.0),
            ]);
        }

        let result = validator.validate(&expr);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.rule, ValidationRule::ComplexityLimits);
            assert!(e.message.contains("nesting depth"));
        }
    }

    #[test]
    fn test_composite_validator_all_pass() {
        let composite = CompositeValidator::new()
            .add_validator(Box::new(TypeSafetyValidator::new()))
            .add_validator(Box::new(ResourceBoundsValidator::new()))
            .add_validator(Box::new(FFIRestrictionsValidator::new()))
            .add_validator(Box::new(ComplexityLimitsValidator::new()));

        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);

        assert!(composite.validate_all(&expr).is_ok());
    }

    #[test]
    fn test_composite_validator_multiple_errors() {
        let composite = CompositeValidator::new()
            .add_validator(Box::new(TypeSafetyValidator::new()))
            .add_validator(Box::new(FFIRestrictionsValidator::new()));

        // This expression has both type safety and FFI issues
        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::String("hello".to_string()),
            LispExpr::List(vec![
                LispExpr::Symbol("rust-unsafe".to_string()),
                LispExpr::String("dangerous".to_string()),
            ]),
        ]);

        let result = composite.validate_all(&expr);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert_eq!(errors.len(), 2);
            assert!(errors.iter().any(|e| e.rule == ValidationRule::TypeSafety));
            assert!(errors.iter().any(|e| e.rule == ValidationRule::FFIRestrictions));
        }
    }

    #[test]
    fn test_infer_type_basic_types() {
        let validator = TypeSafetyValidator::new();
        assert_eq!(validator.infer_type(&LispExpr::Number(42.0)), InferredType::Number);
        assert_eq!(
            validator.infer_type(&LispExpr::String("hello".to_string())),
            InferredType::String
        );
        assert_eq!(validator.infer_type(&LispExpr::Bool(true)), InferredType::Bool);
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            rule: ValidationRule::TypeSafety,
            message: "Type mismatch".to_string(),
            context: Some("line 42".to_string()),
        };
        let display = format!("{}", error);
        assert!(display.contains("TypeSafety"));
        assert!(display.contains("Type mismatch"));
        assert!(display.contains("line 42"));
    }
}
