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
            Some(Token::Quote) => {
                self.advance();
                match self.peek() {
                    None => Err("Expected expression after quote".to_string()),
                    _ => {
                        let expr = self.parse_expression()?;
                        Ok(LispExpr::Quote(Box::new(expr)))
                    }
                }
            },
            Some(Token::Quasiquote) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(LispExpr::Quasiquote(Box::new(expr)))
            },
            Some(Token::Unquote) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(LispExpr::Unquote(Box::new(expr)))
            },
            Some(Token::Splice) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(LispExpr::Splice(Box::new(expr)))
            },
            Some(Token::RightParen) => {
                Err("Unexpected ')' - missing opening parenthesis".to_string())
            },
            None => Err("Unexpected end of input".to_string()),
        }
    }
    
    fn parse_list(&mut self) -> Result<LispExpr, String> {
        self.advance();
        
        // Check for special forms
        if let Some(Token::Symbol(s)) = self.peek() {
            match s.as_str() {
                "defmacro" => return self.parse_defmacro(),
                "quote" => return self.parse_quote_longhand(),
                "quasiquote" => return self.parse_quasiquote_longhand(),
                "unquote" => return self.parse_unquote_longhand(),
                "unquote-splicing" => return self.parse_splice_longhand(),
                _ => {}
            }
        }
        
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

    fn parse_defmacro(&mut self) -> Result<LispExpr, String> {
        // Consume 'defmacro'
        self.advance();
        
        // Parse macro name
        let name = match self.peek() {
            Some(Token::Symbol(s)) => {
                let name = s.clone();
                self.advance();
                name
            },
            _ => return Err("Missing macro name after 'defmacro'".to_string()),
        };
        
        // Parse parameter list
        let parameters = match self.peek() {
            Some(Token::LeftParen) => {
                self.parse_parameter_list()?
            },
            _ => return Err("Missing parameter list for macro definition".to_string()),
        };
        
        // Parse macro body
        let body = match self.peek() {
            Some(Token::RightParen) => return Err("Missing macro body".to_string()),
            Some(_) => {
                let body = self.parse_expression()?;
                Box::new(body)
            },
            None => return Err("Missing macro body".to_string()),
        };
        
        // Consume closing paren
        match self.peek() {
            Some(Token::RightParen) => {
                self.advance();
            },
            _ => return Err("Expected ')' after macro definition".to_string()),
        }
        
        Ok(LispExpr::Macro { name, parameters, body })
    }
    
    fn parse_parameter_list(&mut self) -> Result<Vec<String>, String> {
        // Consume opening paren
        self.advance();
        
        let mut parameters = Vec::new();
        
        while let Some(token) = self.peek() {
            match token {
                Token::RightParen => {
                    self.advance();
                    return Ok(parameters);
                },
                Token::Symbol(s) => {
                    parameters.push(s.clone());
                    self.advance();
                },
                _ => return Err("Expected symbol in parameter list".to_string()),
            }
        }
        
        Err("Unclosed parameter list - missing ')'".to_string())
    }
    
    fn parse_quote_longhand(&mut self) -> Result<LispExpr, String> {
        self.advance(); // consume 'quote'
        
        let expr = self.parse_expression()?;
        
        // Consume closing paren
        match self.peek() {
            Some(Token::RightParen) => {
                self.advance();
                Ok(LispExpr::Quote(Box::new(expr)))
            },
            _ => Err("Expected ')' after quote expression".to_string()),
        }
    }
    
    fn parse_quasiquote_longhand(&mut self) -> Result<LispExpr, String> {
        self.advance(); // consume 'quasiquote'
        
        let expr = self.parse_expression()?;
        
        // Consume closing paren
        match self.peek() {
            Some(Token::RightParen) => {
                self.advance();
                Ok(LispExpr::Quasiquote(Box::new(expr)))
            },
            _ => Err("Expected ')' after quasiquote expression".to_string()),
        }
    }
    
    fn parse_unquote_longhand(&mut self) -> Result<LispExpr, String> {
        self.advance(); // consume 'unquote'
        
        let expr = self.parse_expression()?;
        
        // Consume closing paren
        match self.peek() {
            Some(Token::RightParen) => {
                self.advance();
                Ok(LispExpr::Unquote(Box::new(expr)))
            },
            _ => Err("Expected ')' after unquote expression".to_string()),
        }
    }
    
    fn parse_splice_longhand(&mut self) -> Result<LispExpr, String> {
        self.advance(); // consume 'unquote-splicing'
        
        let expr = self.parse_expression()?;
        
        // Consume closing paren
        match self.peek() {
            Some(Token::RightParen) => {
                self.advance();
                Ok(LispExpr::Splice(Box::new(expr)))
            },
            _ => Err("Expected ')' after unquote-splicing expression".to_string()),
        }
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

    #[test]
    fn test_parse_basic_defmacro() {
        let tokens = tokenize("(defmacro when (condition) body)").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Macro { name, parameters, body } => {
                assert_eq!(name, "when");
                assert_eq!(parameters, &vec!["condition".to_string()]);
                assert_eq!(**body, LispExpr::Symbol("body".to_string()));
            },
            _ => panic!("Expected Macro variant"),
        }
    }

    #[test]
    fn test_parse_defmacro_with_rest_params() {
        let tokens = tokenize("(defmacro when (condition &rest body) nil)").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Macro { name, parameters, body } => {
                assert_eq!(name, "when");
                assert_eq!(parameters, &vec!["condition".to_string(), "&rest".to_string(), "body".to_string()]);
                assert_eq!(**body, LispExpr::Nil);
            },
            _ => panic!("Expected Macro variant"),
        }
    }

    #[test]
    fn test_parse_defmacro_with_complex_body() {
        let tokens = tokenize("(defmacro when (condition &rest body) (if condition (progn body) nil))").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Macro { name, parameters, body } => {
                assert_eq!(name, "when");
                assert_eq!(parameters, &vec!["condition".to_string(), "&rest".to_string(), "body".to_string()]);
                // Body should be a list representing (if condition (progn body) nil)
                match body.as_ref() {
                    LispExpr::List(elements) => {
                        assert_eq!(elements.len(), 4);
                        assert_eq!(elements[0], LispExpr::Symbol("if".to_string()));
                    },
                    _ => panic!("Expected List for macro body"),
                }
            },
            _ => panic!("Expected Macro variant"),
        }
    }

    #[test]
    fn test_parse_defmacro_error_missing_name() {
        let tokens = tokenize("(defmacro)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing macro name"));
    }

    #[test]
    fn test_parse_defmacro_error_missing_params() {
        let tokens = tokenize("(defmacro test)").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing parameter list"));
    }

    #[test]
    fn test_parse_defmacro_error_missing_body() {
        let tokens = tokenize("(defmacro test ())").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing macro body"));
    }

    #[test]
    fn test_parse_quote_shorthand() {
        let tokens = tokenize("'x").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quote(expr) => {
                assert_eq!(**expr, LispExpr::Symbol("x".to_string()));
            },
            _ => panic!("Expected Quote variant"),
        }
    }

    #[test]
    fn test_parse_quote_longhand() {
        let tokens = tokenize("(quote x)").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quote(expr) => {
                assert_eq!(**expr, LispExpr::Symbol("x".to_string()));
            },
            _ => panic!("Expected Quote variant"),
        }
    }

    #[test]
    fn test_parse_quasiquote_shorthand() {
        let tokens = tokenize("`(+ ,x 2)").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quasiquote(expr) => {
                match expr.as_ref() {
                    LispExpr::List(elements) => {
                        assert_eq!(elements.len(), 3);
                        assert_eq!(elements[0], LispExpr::Symbol("+".to_string()));
                        // Check that ,x becomes an Unquote
                        match &elements[1] {
                            LispExpr::Unquote(unquoted) => {
                                assert_eq!(**unquoted, LispExpr::Symbol("x".to_string()));
                            },
                            _ => panic!("Expected Unquote variant for ,x"),
                        }
                        assert_eq!(elements[2], LispExpr::Number(2.0));
                    },
                    _ => panic!("Expected List in quasiquote"),
                }
            },
            _ => panic!("Expected Quasiquote variant"),
        }
    }

    #[test]
    fn test_parse_quasiquote_longhand() {
        let tokens = tokenize("(quasiquote (+ (unquote x) 2))").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quasiquote(expr) => {
                match expr.as_ref() {
                    LispExpr::List(elements) => {
                        assert_eq!(elements.len(), 3);
                        assert_eq!(elements[0], LispExpr::Symbol("+".to_string()));
                        match &elements[1] {
                            LispExpr::Unquote(unquoted) => {
                                assert_eq!(**unquoted, LispExpr::Symbol("x".to_string()));
                            },
                            _ => panic!("Expected Unquote variant"),
                        }
                        assert_eq!(elements[2], LispExpr::Number(2.0));
                    },
                    _ => panic!("Expected List in quasiquote"),
                }
            },
            _ => panic!("Expected Quasiquote variant"),
        }
    }

    #[test]
    fn test_parse_splice_shorthand() {
        let tokens = tokenize("`(list ,@items)").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quasiquote(expr) => {
                match expr.as_ref() {
                    LispExpr::List(elements) => {
                        assert_eq!(elements.len(), 2);
                        assert_eq!(elements[0], LispExpr::Symbol("list".to_string()));
                        match &elements[1] {
                            LispExpr::Splice(spliced) => {
                                assert_eq!(**spliced, LispExpr::Symbol("items".to_string()));
                            },
                            _ => panic!("Expected Splice variant for ,@items"),
                        }
                    },
                    _ => panic!("Expected List in quasiquote"),
                }
            },
            _ => panic!("Expected Quasiquote variant"),
        }
    }

    #[test]
    fn test_parse_splice_longhand() {
        let tokens = tokenize("(quasiquote (list (unquote-splicing items)))").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quasiquote(expr) => {
                match expr.as_ref() {
                    LispExpr::List(elements) => {
                        assert_eq!(elements.len(), 2);
                        assert_eq!(elements[0], LispExpr::Symbol("list".to_string()));
                        match &elements[1] {
                            LispExpr::Splice(spliced) => {
                                assert_eq!(**spliced, LispExpr::Symbol("items".to_string()));
                            },
                            _ => panic!("Expected Splice variant"),
                        }
                    },
                    _ => panic!("Expected List in quasiquote"),
                }
            },
            _ => panic!("Expected Quasiquote variant"),
        }
    }

    #[test]
    fn test_parse_nested_quotes() {
        let tokens = tokenize("'`(+ ,x)").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            LispExpr::Quote(expr) => {
                match expr.as_ref() {
                    LispExpr::Quasiquote(inner) => {
                        match inner.as_ref() {
                            LispExpr::List(elements) => {
                                assert_eq!(elements.len(), 2);
                                assert_eq!(elements[0], LispExpr::Symbol("+".to_string()));
                                match &elements[1] {
                                    LispExpr::Unquote(unquoted) => {
                                        assert_eq!(**unquoted, LispExpr::Symbol("x".to_string()));
                                    },
                                    _ => panic!("Expected Unquote variant"),
                                }
                            },
                            _ => panic!("Expected List in nested quasiquote"),
                        }
                    },
                    _ => panic!("Expected Quasiquote in quote"),
                }
            },
            _ => panic!("Expected Quote variant"),
        }
    }

    #[test]
    fn test_parse_quote_error_missing_expression() {
        let tokens = tokenize("'").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected expression after quote"));
    }
}