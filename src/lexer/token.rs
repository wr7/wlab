use std::fmt::Display;

use super::BracketType;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token<'a> {
    OpenBracket(BracketType),
    CloseBracket(BracketType),
    Identifier(&'a str),
    StringLiteral(String),
    Plus,
    Minus,
    Asterisk,
    Slash,
    Greater,
    Less,
    Dot,
    Comma,
    Colon,
    Semicolon,
    EqualSign,
    Bang,
    Arrow,
    Or,
    And,
    DoubleEqual,
    NotEqual,
    GreaterOrEqual,
    LessOrEqual,
    DoubleColon,
    HashTag,
}

/// Shorthand macro for `Token` literals.
#[macro_export]
macro_rules! T {
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
    (">") => {
        $crate::lexer::Token::Greater
    };
    ("<") => {
        $crate::lexer::Token::Less
    };
    (".") => {
        $crate::lexer::Token::Dot
    };
    (",") => {
        $crate::lexer::Token::Comma
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
    (":") => {
        $crate::lexer::Token::Colon
    };
    (";") => {
        $crate::lexer::Token::Semicolon
    };
    ("=") => {
        $crate::lexer::Token::EqualSign
    };
    ("!") => {
        $crate::lexer::Token::Bang
    };
    ("->") => {
        $crate::lexer::Token::Arrow
    };
    ("||") => {
        $crate::lexer::Token::Or
    };
    ("&&") => {
        $crate::lexer::Token::And
    };
    ("==") => {
        $crate::lexer::Token::DoubleEqual
    };
    ("!=") => {
        $crate::lexer::Token::NotEqual
    };
    (">=") => {
        $crate::lexer::Token::GreaterOrEqual
    };
    ("<=") => {
        $crate::lexer::Token::LessOrEqual
    };
    ("::") => {
        $crate::lexer::Token::DoubleColon
    };
    ("#") => {
        $crate::lexer::Token::HashTag
    };
    ($ident:literal) => {
        $crate::lexer::Token::Identifier($ident)
    };
}

impl<'a> Token<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Token::Identifier(ident) => ident,
            Token::StringLiteral(lit) => lit,
            T!("+") => "+",
            T!("-") => "-",
            T!("*") => "*",
            T!("/") => "/",
            T!(">") => ">",
            T!("<") => "<",
            T!(".") => ".",
            T!(",") => ",",
            T!("(") => "(",
            T!(")") => ")",
            T!("[") => "[",
            T!("]") => "]",
            T!("{") => "{",
            T!("}") => "}",
            T!(":") => ":",
            T!(";") => ";",
            T!("!") => "!",
            T!("=") => "=",
            T!("->") => "->",
            T!("||") => "||",
            T!("&&") => "&&",
            T!("==") => "==",
            T!("!=") => "!=",
            T!(">=") => ">=",
            T!("<=") => "<=",
            T!("::") => "::",
            T!("#") => "#",
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}
