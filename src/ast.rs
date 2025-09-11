#[derive(Debug, Clone, PartialEq)]
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
}