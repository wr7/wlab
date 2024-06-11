use crate::{
    diagnostic as d,
    error_handling::{Diagnostic, Hint, Spanned, WLangError},
    T,
};

use wutil::prelude::*;

mod token;

pub use token::Token;
use wutil::Span;

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
    InvalidToken(Span),
    UnclosedString(Span),
    InvalidEscape(Span),
}

impl WLangError for LexerError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        match self {
            LexerError::InvalidToken(span) => d! {
                format!("invalid token `{}`", &code[*span]),
                [Hint::new_error("", *span)],
            },
            LexerError::UnclosedString(span) => d! {
                "unclosed string",
                [Hint::new_error("string starts here", *span)],
            },
            LexerError::InvalidEscape(span) => d! {
                format!("invalid escape sequence \"\\{}\"", &code[*span]),
                [Hint::new_error("", *span)],
            },
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
    type Item = Result<Spanned<Token<'a>>, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (byte_index, char) = self.chars.next()?;

            if char.is_ascii_whitespace() {
                continue;
            }

            if char.is_ascii_alphanumeric() || char == '_' {
                let ident_span = self.lex_ident(byte_index);

                return Some(Ok(Spanned(
                    Token::Identifier(&self.input[ident_span]),
                    ident_span,
                )));
            }

            if char == '"' {
                return Some(self.lex_string(byte_index));
            }

            return Some(self.lex_symbol(byte_index, char));
        }
    }
}

impl<'a> Lexer<'a> {
    fn lex_ident(&mut self, ident_start: usize) -> Span {
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

        (ident_start..ident_end).into()
    }

    fn lex_string(&mut self, string_start: usize) -> Result<Spanned<Token<'a>>, LexerError> {
        let mut string = String::new();
        let string_end;

        loop {
            let Some((byte_index, char)) = self.chars.next() else {
                return Err(LexerError::UnclosedString(Span::at(string_start)));
            };

            match char {
                '"' => {
                    string_end = byte_index;
                    break;
                }
                '\\' => {
                    let Some((byte_index, char)) = self.chars.next() else {
                        continue; // Triggers an "unclosed string" error
                    };

                    let char_to_add = match char {
                        'n' => '\n',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => {
                            return Err(LexerError::InvalidEscape(
                                Span::at(byte_index).with_len(char.len_utf8()),
                            ))
                        }
                    };

                    string.push(char_to_add);
                }
                _ => string.push(char),
            }
        }

        return Ok(Spanned(
            Token::StringLiteral(string),
            Span::at(string_start).with_end(string_end + 1),
        ));
    }

    fn lex_symbol(
        &mut self,
        byte_index: usize,
        char: char,
    ) -> Result<Spanned<Token<'a>>, LexerError> {
        if char == '-' && self.chars.clone().next().is_some_and(|c| c.1 == '>') {
            self.chars.next();
            return Ok(Spanned(T!("->"), Span::at(byte_index).with_len(2)));
        }

        Ok(Spanned(
            match char {
                '+' => T!("+"),
                '-' => T!("-"),
                '*' => T!("*"),
                '/' => T!("/"),
                '.' => T!("."),
                ',' => T!(","),
                '(' => T!("("),
                ')' => T!(")"),
                '[' => T!("["),
                ']' => T!("]"),
                '{' => T!("{"),
                '}' => T!("}"),
                ':' => T!(":"),
                ';' => T!(";"),
                '=' => T!("="),
                _ => {
                    return Err(LexerError::InvalidToken(
                        self.input.char_span(byte_index).unwrap(),
                    ));
                }
            },
            Span::at(byte_index).with_len(1),
        ))
    }
}
