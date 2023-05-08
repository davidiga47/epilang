use crate::token::Token;
use crate::token::Operand;
use crate::token::Operator;

pub struct LexicalError {
    pub msg: String
}

pub fn tokenize(text: String) -> Result<Vec<Token>, LexicalError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer = String::from("");

    // If callable is true, then `(` is interpreted as the start of a function call and `[` is interpreted as the
    // start of a list selection. Otherwise `(` is a regular round bracket and `[` is the start of a list definition
    let mut callable = false;

    let mut chars = text.chars().peekable();
    loop {
        match chars.next() {
            Option::None => break,

            Option::Some(' ' | '\n' | '\t') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
            },

            Option::Some('(') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token: Token = if callable {Token::FunctionCallOpen} else {Token::RoundBracketOpen};
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('[') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token: Token = if callable {Token::ListSelectionOpen} else {Token::SquareBracketOpen};
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some(c) if [
                ';', ',', '+', '-', '*', ']', '{', '}', ')', '%'
            ].contains(&c) => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = make_token(&c.to_string())?;
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('=') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = match chars.peek() {
                    Option::Some('=') => {
                        chars.next();
                        Token::Operator(Operator::Eq)
                    },
                    _ => Token::Operator(Operator::Assign)
                };
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('>') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = match chars.peek() {
                    Option::Some('=') => {
                        chars.next();
                        Token::Operator(Operator::Gte)
                    },
                    _ => Token::Operator(Operator::Gt)
                };
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('<') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = match chars.peek() {
                    Option::Some('=') => {
                        chars.next();
                        Token::Operator(Operator::Lte)
                    },
                    _ => Token::Operator(Operator::Lt)
                };
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('!') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = match chars.peek() {
                    Option::Some('=') => {
                        chars.next();
                        Token::Operator(Operator::Neq)
                    },
                    _ =>Token::Operator(Operator::Not)
                };
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('&') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = match chars.next() {
                    Option::Some('&') => {
                        Token::Operator(Operator::And)
                    },
                    _ => return Result::Err(LexicalError { msg: String::from("Unexpected character `&`") })
                };
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('|') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                let token = match chars.next() {
                    Option::Some('|') => {
                        Token::Operator(Operator::Or)
                    },
                    _ => return Result::Err(LexicalError { msg: String::from("Unexpected character `|`") })
                };
                callable = token.is_callable();
                tokens.push(token)
            },

            Option::Some('"') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                loop {
                    match chars.next() {
                        Option::Some('"') => break,
                        Option::Some('\\') => {
                            match chars.peek() {
                                Option::Some('n') => {
                                    chars.next();
                                    buffer.push('\n')
                                }
                                Option::Some('"') => {
                                    chars.next();
                                    buffer.push('"')
                                },
                                _ => buffer.push('\\')
                            };
                        }
                        Option::Some(c) => buffer.push(c),
                        Option::None => return Result::Err(LexicalError { msg: String::from("Unclosed string") })
                    }
                }
                let token = Token::Operand(Operand::Str(buffer.clone()));
                buffer.clear();
                callable = token.is_callable();
                tokens.push(token);
            },

            Option::Some('/') => {
                flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
                match chars.next() {
                    Option::Some('/') => { // single line comment
                        loop {
                            match chars.next() {
                                Option::Some('\n') => break,
                                _ => ()
                            }
                        }
                    },
                    Option::Some('*') => { /* multiline comment */
                        loop {
                            match chars.next() {
                                Option::None => return Result::Err(LexicalError { msg: String::from("Unclosed comment") }),
                                Option::Some('*') => {
                                    match chars.next() {
                                        Option::None => return Result::Err(LexicalError { msg: String::from("Unclosed comment") }),
                                        Option::Some('/') => break,
                                        _ => ()
                                    }
                                },
                                _ => ()
                            }
                        }
                    },
                    _ => {
                        let token = Token::Operator(Operator::Div);
                        callable = token.is_callable();
                        tokens.push(token)
                    }
                }
            },

            Option::Some(c) => buffer.push(c)
        }
    };
    flush_buffer(&mut buffer, &mut tokens, &mut callable)?;
    tokens.reverse();
    Result::Ok(tokens)
}

fn flush_buffer(buffer: &mut String, tokens: &mut Vec<Token>, callable: &mut bool) -> Result<(), LexicalError> {
    if !buffer.is_empty() {
        let token: Token = make_token(&buffer)?;
        buffer.clear();
        *callable = token.is_callable();
        tokens.push(token);
    };
    Result::Ok(())
}

fn make_token(word: &String) -> Result<Token, LexicalError> {
    let token = match word.as_str() {
        "true" => Token::Operand(Operand::Bool(true)),
        "false" => Token::Operand(Operand::Bool(false)),
        "null" => Token::Operand(Operand::Null),
        ";" => Token::Operator(Operator::Seq),
        "=" => Token::Operator(Operator::Assign),
        "&&" => Token::Operator(Operator::And),
        "||" => Token::Operator(Operator::Or),
        "!" => Token::Operator(Operator::Not),
        "==" => Token::Operator(Operator::Eq),
        "!=" => Token::Operator(Operator::Neq),
        "<" => Token::Operator(Operator::Lt),
        ">" => Token::Operator(Operator::Gt),
        "+" => Token::Operator(Operator::Sum),
        "-" => Token::Operator(Operator::Sub),
        "*" => Token::Operator(Operator::Mul),
        "/" => Token::Operator(Operator::Div),
        "%" => Token::Operator(Operator::Mod),
        "let" => Token::Let,
        "fn" => Token::Fn,
        "while" => Token::While,
        "if" => Token::If,
        "else" => Token:: Else,
        "try" => Token::Try,
        "catch" => Token::Catch,
        "throw" => Token::Operator(Operator::Throw),
        "callcc" => Token::Callcc,
        "in" => Token::In,
        "(" => Token::RoundBracketOpen,
        ")" => Token::RoundBracketClosed,
        "[" => Token::SquareBracketOpen,
        "]" => Token::SquareBracketClosed,
        "{" => Token::CurlyBracketOpen,
        "}" => Token::CurlyBracketClosed,
        "," => Token::Comma,
        s => match s.parse::<i32>() {
            Result::Ok(i) => Token::Operand(Operand::Int(i)),
            Result::Err(_) => Token::Operand(Operand::Var(String::from(s)))
        }
    };
    Result::Ok(token)
}
