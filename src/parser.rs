use std::ops::Deref;

use crate::{
    error_handling::{Spanned, WLangError},
    lexer::{BracketType, Token},
};

mod rules;

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Let(&'a str, Box<Expression<'a>>),
    Assign(&'a str, Box<Expression<'a>>),
    Function(&'a str, Vec<Statement<'a>>),
}

impl<'a> From<Expression<'a>> for Statement<'a> {
    fn from(expr: Expression<'a>) -> Self {
        Statement::Expression(expr)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Identifier(&'a str),
    BinaryOperator(Box<Self>, OpCode, Box<Self>),
    CompoundExpression(Vec<Statement<'a>>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Plus,
    Minus,
    Asterisk,
    Slash,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidExpression,
    UnmatchedBracket,
    MismatchedBracket(BracketType), // TODO: include position of opening bracket
    ExpectedParameters,
    ExpectedBody,
    ExpectedExpression,
    ExpectedToken(Token<'static>),
}

impl WLangError for ParseError {
    fn get_msg(error: &Spanned<Self>, code: &str) -> std::borrow::Cow<'static, str> {
        match error.deref() {
            ParseError::InvalidExpression => {
                format!("Invalid expression `{}`", &code[error.1.clone()]).into()
            }
            ParseError::UnmatchedBracket => {
                format!("unmatched bracket `{}`", &code[error.1.clone()]).into()
            }
            ParseError::MismatchedBracket(bt) => format!(
                "mismatched bracket; expected {}, got `{}`",
                Token::CloseBracket(*bt),
                &code[error.1.clone()]
            )
            .into(),
            ParseError::ExpectedParameters => "expected function parameters `()`".into(),
            ParseError::ExpectedBody => "expected function body".into(),
            ParseError::ExpectedExpression => "expected expression".into(),
            ParseError::ExpectedToken(tok) => format!("expected token {}", tok).into(),
        }
    }
}

pub fn parse<'a>(
    tokens: &'a [Spanned<Token<'a>>],
) -> Result<Vec<Statement<'a>>, Spanned<ParseError>> {
    rules::parse_statement_list(tokens)
}
