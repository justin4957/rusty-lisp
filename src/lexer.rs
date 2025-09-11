#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    Number(f64),
    Symbol(String),
    String(String),
    Bool(bool),
    Nil,
    Quote,          // '
    Quasiquote,     // `
    Unquote,        // ,
    Splice,         // ,@
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    
    while let Some((pos, ch)) = chars.next() {
        match ch {
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '"' => {
                let mut string_content = String::new();
                let mut escaped = false;
                
                while let Some((_, ch)) = chars.next() {
                    if escaped {
                        match ch {
                            'n' => string_content.push('\n'),
                            't' => string_content.push('\t'),
                            'r' => string_content.push('\r'),
                            '\\' => string_content.push('\\'),
                            '"' => string_content.push('"'),
                            _ => {
                                string_content.push('\\');
                                string_content.push(ch);
                            }
                        }
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        break;
                    } else {
                        string_content.push(ch);
                    }
                }
                tokens.push(Token::String(string_content));
            },
            ';' => {
                while let Some((_, ch)) = chars.peek() {
                    if *ch == '\n' {
                        break;
                    }
                    chars.next();
                }
            },
            '\'' => tokens.push(Token::Quote),
            '`' => tokens.push(Token::Quasiquote),
            ',' => {
                // Check for ,@ (splice)
                if let Some((_, '@')) = chars.peek() {
                    chars.next(); // consume @
                    tokens.push(Token::Splice);
                } else {
                    tokens.push(Token::Unquote);
                }
            },
            ch if ch.is_whitespace() => {},
            ch if ch.is_ascii_digit() || ch == '-' || ch == '+' => {
                let start_pos = pos;
                let mut number_str = String::new();
                number_str.push(ch);
                
                let mut has_dot = false;
                while let Some((_, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() {
                        number_str.push(*next_ch);
                        chars.next();
                    } else if *next_ch == '.' && !has_dot {
                        has_dot = true;
                        number_str.push(*next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                if number_str.len() == 1 && (ch == '-' || ch == '+') {
                    tokens.push(Token::Symbol(number_str));
                } else {
                    match number_str.parse::<f64>() {
                        Ok(num) => tokens.push(Token::Number(num)),
                        Err(_) => return Err(format!("Invalid number at position {}: {}", start_pos, number_str)),
                    }
                }
            },
            _ => {
                let mut symbol = String::new();
                symbol.push(ch);
                
                while let Some((_, next_ch)) = chars.peek() {
                    if next_ch.is_whitespace() || *next_ch == '(' || *next_ch == ')' || *next_ch == '"' || 
                       *next_ch == '\'' || *next_ch == '`' || *next_ch == ',' {
                        break;
                    }
                    symbol.push(*next_ch);
                    chars.next();
                }
                
                match symbol.as_str() {
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    "nil" => tokens.push(Token::Nil),
                    _ => tokens.push(Token::Symbol(symbol)),
                }
            }
        }
    }
    
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let input = "(+ 1 2)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::LeftParen,
            Token::Symbol("+".to_string()),
            Token::Number(1.0),
            Token::Number(2.0),
            Token::RightParen,
        ]);
    }
    
    #[test]
    fn test_string_tokens() {
        let input = r#"("hello world" "with\nnewline")"#;
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::LeftParen,
            Token::String("hello world".to_string()),
            Token::String("with\nnewline".to_string()),
            Token::RightParen,
        ]);
    }
    
    #[test]
    fn test_boolean_and_nil() {
        let input = "(true false nil)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::LeftParen,
            Token::Bool(true),
            Token::Bool(false),
            Token::Nil,
            Token::RightParen,
        ]);
    }

    #[test]
    fn test_quote_tokens() {
        let input = "'(+ 1 2)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::Quote,
            Token::LeftParen,
            Token::Symbol("+".to_string()),
            Token::Number(1.0),
            Token::Number(2.0),
            Token::RightParen,
        ]);
    }

    #[test]
    fn test_quasiquote_tokens() {
        let input = "`(+ ,x ,(* 2 3))";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::Quasiquote,
            Token::LeftParen,
            Token::Symbol("+".to_string()),
            Token::Unquote,
            Token::Symbol("x".to_string()),
            Token::Unquote,
            Token::LeftParen,
            Token::Symbol("*".to_string()),
            Token::Number(2.0),
            Token::Number(3.0),
            Token::RightParen,
            Token::RightParen,
        ]);
    }

    #[test]
    fn test_splice_tokens() {
        let input = "`(list ,@items)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::Quasiquote,
            Token::LeftParen,
            Token::Symbol("list".to_string()),
            Token::Splice,
            Token::Symbol("items".to_string()),
            Token::RightParen,
        ]);
    }

    #[test]
    fn test_mixed_quote_tokens() {
        let input = "'x `(+ ,a ,@b)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![
            Token::Quote,
            Token::Symbol("x".to_string()),
            Token::Quasiquote,
            Token::LeftParen,
            Token::Symbol("+".to_string()),
            Token::Unquote,
            Token::Symbol("a".to_string()),
            Token::Splice,
            Token::Symbol("b".to_string()),
            Token::RightParen,
        ]);
    }
}