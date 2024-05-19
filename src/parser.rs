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
    MismatchedBrackets(Span, Span), // TODO: include position of opening bracket
}

impl WLangError for ParseError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        let mut diagnostic = match self {
            ParseError::InvalidExpression(span) => Diagnostic {
                msg: "invalid expression".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::UnmatchedBracket(span) => Diagnostic {
                msg: format!("unmatched bracket `{}`", &code[span.clone()]).into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedParameters(span) => Diagnostic {
                msg: "expected function parameters `()`".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedBody(span) => Diagnostic {
                msg: "expected function body".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedExpression(span) => Diagnostic {
                msg: "expected expression".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedToken(span, tok) => Diagnostic {
                msg: format!("expected token {}", tok).into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::MismatchedBrackets(opening, closing) => Diagnostic {
                msg: "mismatched brackets".into(),
                hints: vec![
                    Hint::new_error("opening bracket here", opening.clone()),
                    Hint::new_error("closing bracket here", closing.clone()),
                ],
            },
        };

        diagnostic.msg = format!("Error while parsing code: {}", &diagnostic.msg).into();

        diagnostic
    }
}

pub fn parse<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Result<Vec<Statement<'a>>, ParseError> {
    rules::parse_statement_list(tokens)
}
