use std::fmt::Display;

use crate::{error_handling::Spanned, lexer::Token};

mod error;
mod rules;
mod util;

pub use error::ParseError;
use wutil::Span;

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Let(&'a str, Box<Spanned<Expression<'a>>>),
    Assign(&'a str, Box<Spanned<Expression<'a>>>),
    Function {
        name: &'a str,
        params: Vec<(&'a str, Spanned<&'a str>)>,
        return_type: Option<Spanned<&'a str>>,
        body: Spanned<CodeBlock<'a>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Identifier(&'a str),
    Literal(Literal<'a>),
    BinaryOperator(Box<Spanned<Self>>, OpCode, Box<Spanned<Self>>),
    CompoundExpression(CodeBlock<'a>),
    FunctionCall(&'a str, Vec<Spanned<Expression<'a>>>),
    If {
        condition: Box<Spanned<Self>>,
        block: CodeBlock<'a>,
        else_block: Option<CodeBlock<'a>>,
    },
}

impl<'a> From<Expression<'a>> for Statement<'a> {
    fn from(expr: Expression<'a>) -> Self {
        Statement::Expression(expr)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CodeBlock<'a> {
    pub body: Vec<Spanned<Statement<'a>>>,
    pub trailing_semicolon: Option<Span>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literal<'a> {
    Number(&'a str),
    String(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Or,
    And,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OpCode::Plus => "+",
            OpCode::Minus => "-",
            OpCode::Asterisk => "*",
            OpCode::Slash => "/",
            OpCode::Or => "||",
            OpCode::And => "&&",
            OpCode::Equal => "==",
            OpCode::NotEqual => "!=",
            OpCode::Greater => ">",
            OpCode::Less => "<",
            OpCode::GreaterEqual => ">=",
            OpCode::LessEqual => "<=",
        };

        write!(f, "{str}")
    }
}

pub fn parse<'a>(
    tokens: &'a [Spanned<Token<'a>>],
) -> Result<Vec<Spanned<Statement<'a>>>, ParseError> {
    error::check_brackets(tokens)?;
    rules::parse_statement_list(tokens)
}
