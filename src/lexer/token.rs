use std::{borrow::Cow, fmt::Display};

use super::BracketType;

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

impl<'a> Token<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Token::Identifier(ident) => ident,
            T!("->") => "->",
            T!("+") => "+",
            T!("-") => "-",
            T!("*") => "*",
            T!("/") => "/",
            T!(".") => ".",
            T!("(") => "(",
            T!(")") => ")",
            T!("[") => "[",
            T!("]") => "]",
            T!("{") => "{",
            T!("}") => "}",
            T!(";") => ";",
            T!("=") => "=",
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}