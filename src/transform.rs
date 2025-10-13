use crate::ast::LispExpr;
use std::fmt;

/// Error type for AST transformations
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    /// Transform failed with a custom error message
    TransformFailed(String),
    /// Invalid AST structure encountered
    InvalidAst(String),
    /// Plugin not found in registry
    PluginNotFound(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransformError::TransformFailed(msg) => write!(f, "Transform failed: {}", msg),
            TransformError::InvalidAst(msg) => write!(f, "Invalid AST: {}", msg),
            TransformError::PluginNotFound(name) => write!(f, "Plugin not found: {}", name),
        }
    }
}

impl std::error::Error for TransformError {}

/// Trait for AST transformation plugins
///
/// Transforms are applied between parsing and macro expansion, allowing
/// AI agents and tools to modify the AST structure before compilation.
pub trait ASTTransform {
    /// Returns the name of this transform
    fn name(&self) -> &str;

    /// Apply the transformation to the given AST
    ///
    /// # Arguments
    /// * `ast` - Mutable reference to the AST to transform
    ///
    /// # Returns
    /// * `Ok(())` if transformation succeeded
    /// * `Err(TransformError)` if transformation failed
    fn transform(&self, ast: &mut LispExpr) -> Result<(), TransformError>;
}

/// Registry for managing AST transform plugins
pub struct TransformRegistry {
    transforms: Vec<Box<dyn ASTTransform>>,
}

impl TransformRegistry {
    /// Create a new empty transform registry
    pub fn new() -> Self {
        TransformRegistry {
            transforms: Vec::new(),
        }
    }

    /// Register a new transform plugin
    pub fn register(&mut self, transform: Box<dyn ASTTransform>) {
        self.transforms.push(transform);
    }

    /// Apply all registered transforms to the AST in order
    pub fn apply_all(&self, ast: &mut LispExpr) -> Result<(), TransformError> {
        for transform in &self.transforms {
            transform.transform(ast)?;
        }
        Ok(())
    }

    /// Get the number of registered transforms
    pub fn count(&self) -> usize {
        self.transforms.len()
    }

    /// Get the names of all registered transforms
    pub fn transform_names(&self) -> Vec<String> {
        self.transforms.iter().map(|t| t.name().to_string()).collect()
    }
}

impl Default for TransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Echo transform - prints AST structure for debugging
/// Does not modify the AST
pub struct EchoTransform {
    output: std::sync::Arc<std::sync::Mutex<String>>,
}

impl EchoTransform {
    /// Create a new echo transform
    pub fn new() -> Self {
        EchoTransform {
            output: std::sync::Arc::new(std::sync::Mutex::new(String::new())),
        }
    }

    /// Get the captured output from the echo transform
    pub fn get_output(&self) -> String {
        self.output.lock().unwrap().clone()
    }

    fn format_expr(&self, expr: &LispExpr, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        match expr {
            LispExpr::Number(n) => format!("{}Number({})", prefix, n),
            LispExpr::Symbol(s) => format!("{}Symbol({})", prefix, s),
            LispExpr::String(s) => format!("{}String(\"{}\")", prefix, s),
            LispExpr::Bool(b) => format!("{}Bool({})", prefix, b),
            LispExpr::Nil => format!("{}Nil", prefix),
            LispExpr::List(items) => {
                let mut result = format!("{}List[\n", prefix);
                for item in items {
                    result.push_str(&self.format_expr(item, indent + 1));
                    result.push('\n');
                }
                result.push_str(&format!("{}]", prefix));
                result
            }
            LispExpr::Macro { name, parameters, body: _ } => {
                format!("{}Macro(name: {}, parameters: {:?}, body: ...)", prefix, name, parameters)
            }
            LispExpr::MacroCall { name, args } => {
                format!("{}MacroCall(name: {}, args: {} items)", prefix, name, args.len())
            }
            LispExpr::Quote(inner) => {
                format!("{}Quote[\n{}\n{}]", prefix, self.format_expr(inner, indent + 1), prefix)
            }
            LispExpr::Quasiquote(inner) => {
                format!("{}Quasiquote[\n{}\n{}]", prefix, self.format_expr(inner, indent + 1), prefix)
            }
            LispExpr::Unquote(inner) => {
                format!("{}Unquote[\n{}\n{}]", prefix, self.format_expr(inner, indent + 1), prefix)
            }
            LispExpr::Splice(inner) => {
                format!("{}Splice[\n{}\n{}]", prefix, self.format_expr(inner, indent + 1), prefix)
            }
            LispExpr::Gensym(name) => {
                format!("{}Gensym({})", prefix, name)
            }
        }
    }
}

impl Default for EchoTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTTransform for EchoTransform {
    fn name(&self) -> &str {
        "echo"
    }

    fn transform(&self, ast: &mut LispExpr) -> Result<(), TransformError> {
        let formatted = self.format_expr(ast, 0);
        let mut output = self.output.lock().unwrap();
        output.push_str(&formatted);
        output.push('\n');
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_error_display() {
        let err1 = TransformError::TransformFailed("test error".to_string());
        assert_eq!(err1.to_string(), "Transform failed: test error");

        let err2 = TransformError::InvalidAst("bad structure".to_string());
        assert_eq!(err2.to_string(), "Invalid AST: bad structure");

        let err3 = TransformError::PluginNotFound("missing".to_string());
        assert_eq!(err3.to_string(), "Plugin not found: missing");
    }

    #[test]
    fn test_registry_new() {
        let registry = TransformRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = TransformRegistry::new();
        let transform = Box::new(EchoTransform::new());
        registry.register(transform);
        assert_eq!(registry.count(), 1);
        assert_eq!(registry.transform_names(), vec!["echo"]);
    }

    #[test]
    fn test_echo_transform_name() {
        let transform = EchoTransform::new();
        assert_eq!(transform.name(), "echo");
    }

    #[test]
    fn test_echo_transform_simple() {
        let transform = EchoTransform::new();
        let mut ast = LispExpr::Number(42.0);

        let result = transform.transform(&mut ast);
        assert!(result.is_ok());

        let output = transform.get_output();
        assert!(output.contains("Number(42"));
    }

    #[test]
    fn test_echo_transform_list() {
        let transform = EchoTransform::new();
        let mut ast = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);

        let result = transform.transform(&mut ast);
        assert!(result.is_ok());

        let output = transform.get_output();
        assert!(output.contains("List["));
        assert!(output.contains("Symbol(+)"));
        assert!(output.contains("Number(1"));
        assert!(output.contains("Number(2"));
    }

    #[test]
    fn test_echo_transform_does_not_modify_ast() {
        let transform = EchoTransform::new();
        let original = LispExpr::Number(42.0);
        let mut ast = original.clone();

        transform.transform(&mut ast).unwrap();

        assert_eq!(ast, original);
    }

    #[test]
    fn test_registry_apply_all() {
        let mut registry = TransformRegistry::new();
        let transform = Box::new(EchoTransform::new());
        registry.register(transform);

        let mut ast = LispExpr::Number(42.0);
        let result = registry.apply_all(&mut ast);

        assert!(result.is_ok());
        assert_eq!(ast, LispExpr::Number(42.0)); // AST unchanged
    }

    #[test]
    fn test_registry_multiple_transforms() {
        let mut registry = TransformRegistry::new();
        registry.register(Box::new(EchoTransform::new()));
        registry.register(Box::new(EchoTransform::new()));

        assert_eq!(registry.count(), 2);
        assert_eq!(registry.transform_names(), vec!["echo", "echo"]);

        let mut ast = LispExpr::Symbol("test".to_string());
        let result = registry.apply_all(&mut ast);
        assert!(result.is_ok());
    }

    // Test for custom transform implementation
    struct DoubleNumberTransform;

    impl ASTTransform for DoubleNumberTransform {
        fn name(&self) -> &str {
            "double"
        }

        fn transform(&self, ast: &mut LispExpr) -> Result<(), TransformError> {
            match ast {
                LispExpr::Number(n) => {
                    *n *= 2.0;
                    Ok(())
                }
                LispExpr::List(items) => {
                    for item in items.iter_mut() {
                        self.transform(item)?;
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }
    }

    #[test]
    fn test_custom_transform() {
        let transform = DoubleNumberTransform;
        let mut ast = LispExpr::Number(21.0);

        transform.transform(&mut ast).unwrap();

        match ast {
            LispExpr::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_custom_transform_recursive() {
        let transform = DoubleNumberTransform;
        let mut ast = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);

        transform.transform(&mut ast).unwrap();

        match ast {
            LispExpr::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[1] {
                    LispExpr::Number(n) => assert_eq!(*n, 2.0),
                    _ => panic!("Expected Number"),
                }
                match &items[2] {
                    LispExpr::Number(n) => assert_eq!(*n, 4.0),
                    _ => panic!("Expected Number"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_registry_with_custom_transform() {
        let mut registry = TransformRegistry::new();
        registry.register(Box::new(DoubleNumberTransform));

        let mut ast = LispExpr::Number(10.0);
        registry.apply_all(&mut ast).unwrap();

        match ast {
            LispExpr::Number(n) => assert_eq!(n, 20.0),
            _ => panic!("Expected Number"),
        }
    }
}
