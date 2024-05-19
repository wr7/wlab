use std::ops::{Deref, Range};

use crate::{
    error_handling::{Diagnostic, Hint, Spanned, WLangError},
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

type Span = Range<usize>;

#[derive(Debug)]
pub enum ParseError {
    InvalidExpression(Span),
    UnmatchedBracket(Span),
    ExpectedParameters(Span),
    ExpectedBody(Span),
    ExpectedExpression(Span),
    ExpectedToken(Span, Token<'static>),
    MismatchedBracket(Span, BracketType), // TODO: include position of opening bracket
}

impl WLangError for ParseError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        match self {
            ParseError::InvalidExpression(span) => Diagnostic {
                msg: "Invalid expression".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::UnmatchedBracket(span) => Diagnostic {
                msg: format!("Unmatched bracket `{}`", &code[span.clone()]).into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedParameters(span) => Diagnostic {
                msg: "Expected function parameters `()`".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedBody(span) => Diagnostic {
                msg: "Expected function body".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedExpression(span) => Diagnostic {
                msg: "Expected expression".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedToken(span, tok) => Diagnostic {
                msg: format!("Expected token {}", tok).into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::MismatchedBracket(span, bt) => Diagnostic {
                msg: format!(
                    "Mismatched bracket; expected {}, got `{}`",
                    Token::CloseBracket(*bt),
                    &code[span.clone()]
                )
                .into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
        }
    }
}

pub fn parse<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Result<Vec<Statement<'a>>, ParseError> {
    rules::parse_statement_list(tokens)
}
