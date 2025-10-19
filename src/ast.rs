use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LispExpr {
    Number(f64),
    Symbol(String),
    String(String),
    List(Vec<LispExpr>),
    Bool(bool),
    Nil,
    Macro {
        name: String,
        parameters: Vec<String>,
        body: Box<LispExpr>,
    },
    MacroCall {
        name: String,
        args: Vec<LispExpr>,
    },
    Quote(Box<LispExpr>),
    Quasiquote(Box<LispExpr>),
    Unquote(Box<LispExpr>),
    Splice(Box<LispExpr>),
    Gensym(String),
}

impl LispExpr {
    pub fn is_atom(&self) -> bool {
        matches!(self, 
            LispExpr::Number(_) | 
            LispExpr::Symbol(_) | 
            LispExpr::String(_) | 
            LispExpr::Bool(_) | 
            LispExpr::Nil |
            LispExpr::Gensym(_)
        )
    }
    
    pub fn is_list(&self) -> bool {
        matches!(self, LispExpr::List(_))
    }
    
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            LispExpr::Symbol(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_list(&self) -> Option<&Vec<LispExpr>> {
        match self {
            LispExpr::List(l) => Some(l),
            _ => None,
        }
    }
    
    pub fn is_macro(&self) -> bool {
        matches!(self, LispExpr::Macro { .. })
    }
    
    pub fn is_macro_call(&self) -> bool {
        matches!(self, LispExpr::MacroCall { .. })
    }
    
    pub fn is_quote_family(&self) -> bool {
        matches!(self, 
            LispExpr::Quote(_) | 
            LispExpr::Quasiquote(_) | 
            LispExpr::Unquote(_) | 
            LispExpr::Splice(_)
        )
    }
    
    pub fn as_macro(&self) -> Option<(&str, &Vec<String>, &LispExpr)> {
        match self {
            LispExpr::Macro { name, parameters, body } => Some((name, parameters, body)),
            _ => None,
        }
    }
    
    pub fn as_macro_call(&self) -> Option<(&str, &Vec<LispExpr>)> {
        match self {
            LispExpr::MacroCall { name, args } => Some((name, args)),
            _ => None,
        }
    }
    
    pub fn as_quote(&self) -> Option<&LispExpr> {
        match self {
            LispExpr::Quote(expr) => Some(expr),
            _ => None,
        }
    }
    
    pub fn as_quasiquote(&self) -> Option<&LispExpr> {
        match self {
            LispExpr::Quasiquote(expr) => Some(expr),
            _ => None,
        }
    }
    
    pub fn as_unquote(&self) -> Option<&LispExpr> {
        match self {
            LispExpr::Unquote(expr) => Some(expr),
            _ => None,
        }
    }
    
    pub fn as_splice(&self) -> Option<&LispExpr> {
        match self {
            LispExpr::Splice(expr) => Some(expr),
            _ => None,
        }
    }
    
    pub fn as_gensym(&self) -> Option<&str> {
        match self {
            LispExpr::Gensym(name) => Some(name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_macro_variants() {
        let macro_def = LispExpr::Macro {
            name: "when".to_string(),
            parameters: vec!["condition".to_string(), "body".to_string()],
            body: Box::new(LispExpr::Symbol("test".to_string())),
        };
        
        assert!(macro_def.is_macro());
        assert!(!macro_def.is_atom());
        
        let (name, params, _body) = macro_def.as_macro().unwrap();
        assert_eq!(name, "when");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "condition");
    }

    #[test]
    fn test_quote_variants() {
        let quote_expr = LispExpr::Quote(Box::new(LispExpr::Symbol("x".to_string())));
        let quasiquote_expr = LispExpr::Quasiquote(Box::new(LispExpr::Symbol("y".to_string())));
        let unquote_expr = LispExpr::Unquote(Box::new(LispExpr::Symbol("z".to_string())));
        let splice_expr = LispExpr::Splice(Box::new(LispExpr::List(vec![])));
        
        assert!(quote_expr.is_quote_family());
        assert!(quasiquote_expr.is_quote_family());
        assert!(unquote_expr.is_quote_family());
        assert!(splice_expr.is_quote_family());
        
        assert!(!quote_expr.is_atom());
        assert!(!quasiquote_expr.is_atom());
    }

    #[test]
    fn test_gensym_variant() {
        let gensym = LispExpr::Gensym("unique_123".to_string());
        
        assert!(gensym.is_atom());
        assert_eq!(gensym.as_gensym().unwrap(), "unique_123");
    }

    #[test]
    fn test_macro_call_variant() {
        let macro_call = LispExpr::MacroCall {
            name: "when".to_string(),
            args: vec![LispExpr::Bool(true), LispExpr::Number(42.0)],
        };

        assert!(macro_call.is_macro_call());
        assert!(!macro_call.is_atom());

        let (name, args) = macro_call.as_macro_call().unwrap();
        assert_eq!(name, "when");
        assert_eq!(args.len(), 2);
    }

    // JSON Serialization Tests
    #[test]
    fn test_json_serialize_basic_types() {
        use serde_json;

        // Number
        let number = LispExpr::Number(42.5);
        let json = serde_json::to_string(&number).unwrap();
        assert!(json.contains("Number"));
        assert!(json.contains("42.5"));

        // Symbol
        let symbol = LispExpr::Symbol("foo".to_string());
        let json = serde_json::to_string(&symbol).unwrap();
        assert!(json.contains("Symbol"));
        assert!(json.contains("foo"));

        // String
        let string = LispExpr::String("hello".to_string());
        let json = serde_json::to_string(&string).unwrap();
        assert!(json.contains("String"));
        assert!(json.contains("hello"));

        // Bool
        let bool_val = LispExpr::Bool(true);
        let json = serde_json::to_string(&bool_val).unwrap();
        assert!(json.contains("Bool"));
        assert!(json.contains("true"));

        // Nil
        let nil = LispExpr::Nil;
        let json = serde_json::to_string(&nil).unwrap();
        assert!(json.contains("Nil"));
    }

    #[test]
    fn test_json_serialize_list() {
        use serde_json;

        let list = LispExpr::List(vec![
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
            LispExpr::Number(3.0),
        ]);

        let json = serde_json::to_string(&list).unwrap();
        assert!(json.contains("List"));
    }

    #[test]
    fn test_json_serialize_macro() {
        use serde_json;

        let macro_def = LispExpr::Macro {
            name: "when".to_string(),
            parameters: vec!["cond".to_string(), "body".to_string()],
            body: Box::new(LispExpr::Symbol("test".to_string())),
        };

        let json = serde_json::to_string(&macro_def).unwrap();
        assert!(json.contains("Macro"));
        assert!(json.contains("when"));
        assert!(json.contains("cond"));
    }

    #[test]
    fn test_json_serialize_quote_family() {
        use serde_json;

        let quote = LispExpr::Quote(Box::new(LispExpr::Symbol("x".to_string())));
        let json = serde_json::to_string(&quote).unwrap();
        assert!(json.contains("Quote"));

        let quasiquote = LispExpr::Quasiquote(Box::new(LispExpr::Number(42.0)));
        let json = serde_json::to_string(&quasiquote).unwrap();
        assert!(json.contains("Quasiquote"));

        let unquote = LispExpr::Unquote(Box::new(LispExpr::Symbol("y".to_string())));
        let json = serde_json::to_string(&unquote).unwrap();
        assert!(json.contains("Unquote"));

        let splice = LispExpr::Splice(Box::new(LispExpr::List(vec![])));
        let json = serde_json::to_string(&splice).unwrap();
        assert!(json.contains("Splice"));
    }

    #[test]
    fn test_json_serialize_gensym() {
        use serde_json;

        let gensym = LispExpr::Gensym("unique_123".to_string());
        let json = serde_json::to_string(&gensym).unwrap();
        assert!(json.contains("Gensym"));
        assert!(json.contains("unique_123"));
    }

    #[test]
    fn test_json_round_trip_basic_types() {
        use serde_json;

        // Number
        let original = LispExpr::Number(42.5);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LispExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);

        // Symbol
        let original = LispExpr::Symbol("test".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LispExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);

        // Bool
        let original = LispExpr::Bool(false);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LispExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);

        // Nil
        let original = LispExpr::Nil;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LispExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_json_round_trip_complex_structures() {
        use serde_json;

        // Nested list with macro call
        let original = LispExpr::List(vec![
            LispExpr::MacroCall {
                name: "when".to_string(),
                args: vec![
                    LispExpr::Bool(true),
                    LispExpr::List(vec![
                        LispExpr::Symbol("+".to_string()),
                        LispExpr::Number(1.0),
                        LispExpr::Number(2.0),
                    ]),
                ],
            },
        ]);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LispExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);

        // Macro with quote
        let original = LispExpr::Macro {
            name: "test_macro".to_string(),
            parameters: vec!["x".to_string()],
            body: Box::new(LispExpr::Quote(Box::new(LispExpr::Symbol("x".to_string())))),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LispExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_json_deserialize_from_custom_format() {
        use serde_json;

        // Test deserializing from a JSON string similar to the example in issue
        let json_str = r#"{"Number":42.0}"#;
        let expr: LispExpr = serde_json::from_str(json_str).unwrap();
        assert_eq!(expr, LispExpr::Number(42.0));

        let json_str = r#"{"Symbol":"test"}"#;
        let expr: LispExpr = serde_json::from_str(json_str).unwrap();
        assert_eq!(expr, LispExpr::Symbol("test".to_string()));
    }

    #[test]
    fn test_json_pretty_print() {
        use serde_json;

        let expr = LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ]);

        let json = serde_json::to_string_pretty(&expr).unwrap();
        assert!(json.contains("List"));
        assert!(json.contains("  ")); // Should have indentation
    }
}