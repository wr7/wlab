use crate::{error_handling::Spanned, lexer::Token};

mod error;
mod rules;
mod util;

pub use error::ParseError;

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Let(&'a str, Box<Spanned<Expression<'a>>>),
    Assign(&'a str, Box<Spanned<Expression<'a>>>),
    Function(
        &'a str,
        Vec<(&'a str, &'a str)>,
        Vec<Spanned<Statement<'a>>>,
    ),
}

impl<'a> From<Expression<'a>> for Statement<'a> {
    fn from(expr: Expression<'a>) -> Self {
        Statement::Expression(expr)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literal<'a> {
    Number(&'a str),
    String(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Identifier(&'a str),
    Literal(Literal<'a>),
    BinaryOperator(Box<Spanned<Self>>, OpCode, Box<Spanned<Self>>),
    CompoundExpression(Vec<Spanned<Statement<'a>>>),
    FunctionCall(&'a str, Vec<Spanned<Expression<'a>>>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Plus,
    Minus,
    Asterisk,
    Slash,
}

pub fn parse<'a>(
    tokens: &'a [Spanned<Token<'a>>],
) -> Result<Vec<Spanned<Statement<'a>>>, ParseError> {
    error::check_brackets(tokens)?;
    rules::parse_statement_list(tokens)
}
