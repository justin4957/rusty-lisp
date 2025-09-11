#[derive(Debug, Clone, PartialEq)]
pub enum LispExpr {
    Number(f64),
    Symbol(String),
    String(String),
    List(Vec<LispExpr>),
    Bool(bool),
    Nil,
}

impl LispExpr {
    pub fn is_atom(&self) -> bool {
        matches!(self, LispExpr::Number(_) | LispExpr::Symbol(_) | LispExpr::String(_) | LispExpr::Bool(_) | LispExpr::Nil)
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
}