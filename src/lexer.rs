use std::fmt::{Debug, Display};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token<'a> {
    OpenBracket(BracketType),
    CloseBracket(BracketType),
    Identifier(&'a str),
    Arrow,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Dot,
    Semicolon,
}

/// Shorthand macro for `Token` literals.
#[macro_export]
macro_rules! T {
    ("->") => {
        Token::Arrow
    };
    ("+") => {
        Token::Plus
    };
    ("-") => {
        Token::Minus
    };
    ("/") => {
        Token::Slash
    };
    ("*") => {
        Token::Asterisk
    };
    (".") => {
        Token::Dot
    };
    ("(") => {
        Token::OpenBracket(BracketType::Parenthesis)
    };
    ("[") => {
        Token::OpenBracket(BracketType::Square)
    };
    ("{") => {
        Token::OpenBracket(BracketType::Curly)
    };
    (")") => {
        Token::CloseBracket(BracketType::Parenthesis)
    };
    ("]") => {
        Token::CloseBracket(BracketType::Square)
    };
    ("}") => {
        Token::CloseBracket(BracketType::Curly)
    };
    (";") => {
        Token::Semicolon
    };
    ($ident:literal) => {
        Token::Identifier($ident)
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BracketType {
    Parenthesis,
    Square,
    Curly,
}

#[derive(Clone)]
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::str::CharIndices<'a>,
}

pub enum LexerError<'a> {
    InvalidToken { input: &'a str, byte_index: usize },
}

impl<'a> Display for LexerError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LexerError::InvalidToken { input, byte_index } = self;
        let char: char = input[*byte_index..].chars().next().unwrap();

        write!(f, "Invalid token {:?} at index {}", char, byte_index)
    }
}

impl<'a> Debug for LexerError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(&self, f)
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        return Self {
            input,
            chars: input.char_indices(),
        };
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexerError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (byte_index, char) = self.chars.next()?;

            if char.is_ascii_whitespace() {
                continue;
            }

            if char == '-' && self.chars.clone().next().is_some_and(|c| c.1 == '>') {
                self.chars.next();
                return Some(Ok(T!("->")));
            }

            if !char.is_ascii_alphabetic() {
                return Some(Ok(match char {
                    '+' => T!("+"),
                    '-' => T!("-"),
                    '*' => T!("*"),
                    '/' => T!("/"),
                    '.' => T!("."),
                    '(' => T!("("),
                    ')' => T!(")"),
                    '[' => T!("["),
                    ']' => T!("]"),
                    '{' => T!("{"),
                    '}' => T!("}"),
                    ';' => T!(";"),
                    _ => {
                        return Some(Err(LexerError::InvalidToken {
                            input: self.input,
                            byte_index,
                        }))
                    }
                }));
            }

            let ident_start = byte_index;
            let ident_end;

            loop {
                let Some((byte_index, char)) = self.chars.clone().next() else {
                    ident_end = self.input.len();
                    break;
                };

                if char.is_ascii_alphanumeric() || char == '_' {
                    self.chars.next();
                } else {
                    ident_end = byte_index;
                    break;
                }
            }

            return Some(Ok(Token::Identifier(&self.input[ident_start..ident_end])));
        }
    }
}
