use crate::ast::LispExpr;
use crate::lexer::Token;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<LispExpr>, String> {
    let mut parser = Parser::new(tokens);
    let mut expressions = Vec::new();
    
    while !parser.is_at_end() {
        expressions.push(parser.parse_expression()?);
    }
    
    Ok(expressions)
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }
    
    fn parse_expression(&mut self) -> Result<LispExpr, String> {
        match self.peek() {
            Some(Token::LeftParen) => self.parse_list(),
            Some(Token::Number(n)) => {
                let num = *n;
                self.advance();
                Ok(LispExpr::Number(num))
            },
            Some(Token::Symbol(s)) => {
                let sym = s.clone();
                self.advance();
                Ok(LispExpr::Symbol(sym))
            },
            Some(Token::String(s)) => {
                let string = s.clone();
                self.advance();
                Ok(LispExpr::String(string))
            },
            Some(Token::Bool(b)) => {
                let boolean = *b;
                self.advance();
                Ok(LispExpr::Bool(boolean))
            },
            Some(Token::Nil) => {
                self.advance();
                Ok(LispExpr::Nil)
            },
            Some(Token::RightParen) => {
                Err("Unexpected ')' - missing opening parenthesis".to_string())
            },
            None => Err("Unexpected end of input".to_string()),
        }
    }
    
    fn parse_list(&mut self) -> Result<LispExpr, String> {
        self.advance();
        
        let mut elements = Vec::new();
        
        while let Some(token) = self.peek() {
            match token {
                Token::RightParen => {
                    self.advance();
                    return Ok(LispExpr::List(elements));
                },
                _ => {
                    elements.push(self.parse_expression()?);
                }
            }
        }
        
        Err("Unclosed list - missing ')'".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    
    #[test]
    fn test_parse_atom() {
        let tokens = tokenize("42").unwrap();
        let ast = parse(tokens).unwrap();
        assert_eq!(ast, vec![LispExpr::Number(42.0)]);
    }
    
    #[test]
    fn test_parse_simple_list() {
        let tokens = tokenize("(+ 1 2)").unwrap();
        let ast = parse(tokens).unwrap();
        assert_eq!(ast, vec![
            LispExpr::List(vec![
                LispExpr::Symbol("+".to_string()),
                LispExpr::Number(1.0),
                LispExpr::Number(2.0),
            ])
        ]);
    }
    
    #[test]
    fn test_parse_nested_list() {
        let tokens = tokenize("(* (+ 1 2) 3)").unwrap();
        let ast = parse(tokens).unwrap();
        assert_eq!(ast, vec![
            LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::List(vec![
                    LispExpr::Symbol("+".to_string()),
                    LispExpr::Number(1.0),
                    LispExpr::Number(2.0),
                ]),
                LispExpr::Number(3.0),
            ])
        ]);
    }
}