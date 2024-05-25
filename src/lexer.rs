use crate::{
    diagnostic,
    error_handling::{Diagnostic, Hint, Spanned, WLangError},
    spanned,
    util::{Span, StrExt},
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
pub struct LexerError {
    span: Span,
}

impl WLangError for LexerError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        diagnostic! {
            format!("invalid token `{}`", &code[self.span.clone()]),
            [Hint::new_error("", self.span)],
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

            if char == '-' && self.chars.clone().next().is_some_and(|c| c.1 == '>') {
                self.chars.next();
                return Some(Ok(spanned!(T!("->"), (byte_index)..(byte_index + 2))));
            }

            if !(char.is_ascii_alphabetic() || char == '_') {
                return Some(Ok(spanned!(
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
                        ';' => T!(";"),
                        '=' => T!("="),
                        _ => {
                            return Some(Err(LexerError {
                                span: self.input.char_range(byte_index).unwrap(),
                            }));
                        }
                    },
                    (byte_index)..(byte_index + 1),
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

            return Some(Ok(spanned!(
                Token::Identifier(&self.input[ident_start..ident_end]),
                (ident_start)..(ident_end),
            )));
        }
    }
}
