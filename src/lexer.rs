use std::fmt::{Debug, Display};

use crate::{
    error_handling::{Spanned, WLangError},
    util::StrExt,
    T,
};

mod token;

pub use token::Token;

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
    type Item = Result<Spanned<Token<'a>>, Spanned<LexerError>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (byte_index, char) = self.chars.next()?;

            if char.is_ascii_whitespace() {
                continue;
            }

            if char == '-' && self.chars.clone().next().is_some_and(|c| c.1 == '>') {
                self.chars.next();
                return Some(Ok(Spanned(T!("->"), byte_index..byte_index + 2)));
            }

            if !(char.is_ascii_alphabetic() || char == '_') {
                return Some(Ok(Spanned(
                    match char {
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
                    },
                    byte_index..byte_index + 1,
                )));
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

            return Some(Ok(Spanned(
                Token::Identifier(&self.input[ident_start..ident_end]),
                ident_start..ident_end,
            )));
        }
    }
}
