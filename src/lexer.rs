use crate::{
    error_handling::{Diagnostic, Spanned},
    T,
};

use wutil::prelude::*;

mod error;
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

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        return Self {
            input,
            chars: input.char_indices(),
        };
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Spanned<Token<'a>>, Diagnostic>;

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

            if char == '/' {
                match self.try_lex_comment() {
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(err) => return Some(Err(err)),
                };
            }

            return Some(self.lex_symbol(byte_index, char));
        }
    }
}

impl<'a> Lexer<'a> {
    fn try_lex_comment(&mut self) -> Result<bool, Diagnostic> {
        let Some(next_char) = self.chars.clone().next() else {
            return Ok(false);
        };

        match next_char.1 {
            '/' => {
                self.chars
                    .by_ref()
                    .take_while(|(_, c)| *c != '\n')
                    .for_each(|_| {});
                Ok(true)
            }
            '*' => {
                self.chars.next();

                let mut previous_char = '\0';
                let mut closed = false;

                for (_, char) in self.chars.by_ref() {
                    if matches!([previous_char, char], ['*', '/']) {
                        closed = true;
                        break;
                    }

                    previous_char = char;
                }

                if !closed {
                    return Err(error::unclosed_comment(
                        Span::at(next_char.0 - 1).with_len(2),
                    ));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

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

    fn lex_string(&mut self, string_start: usize) -> Result<Spanned<Token<'a>>, Diagnostic> {
        let mut string = String::new();
        let string_end;

        loop {
            let Some((byte_index, char)) = self.chars.next() else {
                return Err(error::unclosed_string(Span::at(string_start)));
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
                            let span = self.input.char_span(byte_index).unwrap();

                            return Err(error::invalid_escape(Spanned(&self.input[span], span)));
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
    ) -> Result<Spanned<Token<'a>>, Diagnostic> {
        if let Some(symbol) = self.lex_two_character_symbol(byte_index, char) {
            return Ok(symbol);
        }

        Ok(Spanned(
            match char {
                '+' => T!("+"),
                '-' => T!("-"),
                '*' => T!("*"),
                '/' => T!("/"),
                '>' => T!(">"),
                '<' => T!("<"),
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
                '!' => T!("!"),
                '=' => T!("="),
                '#' => T!("#"),
                _ => {
                    let span = self.input.char_span(byte_index).unwrap();
                    return Err(error::invalid_token(Spanned(&self.input[span], span)));
                }
            },
            Span::at(byte_index).with_len(1),
        ))
    }

    fn lex_two_character_symbol(
        &mut self,
        byte_index: usize,
        char: char,
    ) -> Option<Spanned<Token<'a>>> {
        let next_char = self.chars.clone().next()?.1;

        let symbol = match (char, next_char) {
            ('-', '>') => T!("->"),
            ('|', '|') => T!("||"),
            ('&', '&') => T!("&&"),
            ('=', '=') => T!("=="),
            ('!', '=') => T!("!="),
            ('>', '=') => T!(">="),
            ('<', '=') => T!("<="),
            (':', ':') => T!("::"),
            _ => return None,
        };

        self.chars.next();

        Some(Spanned(symbol, Span::at(byte_index).with_len(2)))
    }
}
