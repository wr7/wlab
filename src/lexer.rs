use std::fmt::{Debug, Display};

use crate::{
    error_handling::{Spanned, WLangError},
    util::StrExt,
};

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
    EqualSign,
}

/// Shorthand macro for `Token` literals.
#[macro_export]
macro_rules! T {
    ("->") => {
        $crate::lexer::Token::Arrow
    };
    ("+") => {
        $crate::lexer::Token::Plus
    };
    ("-") => {
        $crate::lexer::Token::Minus
    };
    ("/") => {
        $crate::lexer::Token::Slash
    };
    ("*") => {
        $crate::lexer::Token::Asterisk
    };
    (".") => {
        $crate::lexer::Token::Dot
    };
    ("(") => {
        $crate::lexer::Token::OpenBracket($crate::lexer::BracketType::Parenthesis)
    };
    ("[") => {
        $crate::lexer::Token::OpenBracket($crate::lexer::BracketType::Square)
    };
    ("{") => {
        $crate::lexer::Token::OpenBracket($crate::lexer::BracketType::Curly)
    };
    (")") => {
        $crate::lexer::Token::CloseBracket($crate::lexer::BracketType::Parenthesis)
    };
    ("]") => {
        $crate::lexer::Token::CloseBracket($crate::lexer::BracketType::Square)
    };
    ("}") => {
        $crate::lexer::Token::CloseBracket($crate::lexer::BracketType::Curly)
    };
    (";") => {
        $crate::lexer::Token::Semicolon
    };
    ("=") => {
        $crate::lexer::Token::EqualSign
    };
    ($ident:literal) => {
        $crate::lexer::Token::Identifier($ident)
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

#[derive(Debug)]
pub enum LexerError {
    InvalidToken,
}

impl WLangError for LexerError {
    fn get_msg(error: &Spanned<Self>, code: &str) -> std::borrow::Cow<'static, str> {
        match error.0 {
            LexerError::InvalidToken => {
                format!("invalid token `{}`", &code[error.1.clone()]).into()
            }
        }
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
    type Item = Result<Token<'a>, Spanned<LexerError>>;

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

            if !(char.is_ascii_alphabetic() || char == '_') {
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
                    '=' => T!("="),
                    _ => {
                        return Some(Err(Spanned(
                            LexerError::InvalidToken,
                            self.input.char_range(byte_index).unwrap(),
                        )));
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
