use std::fmt::Display;

use crate::{error_handling::Spanned as S, lexer::Token};

mod error;
mod rules;
mod util;

pub use error::ParseError;
use wutil::Span;

pub type TokenStream<'src> = [S<Token<'src>>];
pub type Path<'a> = Vec<S<&'a str>>;

pub fn parse_module<'src>(mut tokens: &TokenStream<'src>) -> Result<Module<'src>, ParseError> {
    error::check_brackets(tokens)?;

    let attributes;
    if let Some((attributes_, tokens_)) = rules::try_parse_outer_attributes_from_front(tokens)? {
        tokens = tokens_;
        attributes = attributes_;
    } else {
        attributes = Vec::new();
    }

    let statements = rules::parse_statement_list(tokens)?;
    let functions: Result<Vec<S<Function>>, _> = statements
        .into_iter()
        .map(|S(statement, span)| {
            Function::try_from(statement)
                .map(|s| S(s, span))
                .map_err(|()| ParseError::ExpectedFunction(span))
        })
        .collect();

    let functions = functions?;

    Ok(Module {
        attributes,
        functions,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module<'src> {
    pub attributes: Vec<S<Attribute>>,
    pub functions: Vec<S<Function<'src>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Attribute {
    DeclareCrate(String),
    NoMangle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function<'src> {
    pub name: &'src str,
    pub params: Vec<(&'src str, S<&'src str>)>,
    pub return_type: Option<S<&'src str>>,
    pub attributes: Vec<S<Attribute>>,
    pub visibility: Visibility,
    pub body: S<CodeBlock<'src>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'src> {
    Expression(Expression<'src>),
    Let(&'src str, Box<S<Expression<'src>>>),
    Assign(&'src str, Box<S<Expression<'src>>>),
    Function(Function<'src>),
}

impl<'src> TryFrom<Statement<'src>> for Function<'src> {
    type Error = ();

    fn try_from(stmnt: Statement<'src>) -> Result<Self, Self::Error> {
        if let Statement::Function(f) = stmnt {
            Ok(f)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'src> {
    Identifier(&'src str),
    Literal(Literal<'src>),
    BinaryOperator(Box<S<Self>>, OpCode, Box<S<Self>>),
    CompoundExpression(CodeBlock<'src>),
    FunctionCall(S<Path<'src>>, Vec<S<Expression<'src>>>),
    If {
        condition: Box<S<Self>>,
        block: S<CodeBlock<'src>>,
        else_block: Option<S<CodeBlock<'src>>>,
    },
}

impl<'src> From<Expression<'src>> for Statement<'src> {
    fn from(expr: Expression<'src>) -> Self {
        Statement::Expression(expr)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CodeBlock<'src> {
    pub body: Vec<S<Statement<'src>>>,
    pub trailing_semicolon: Option<Span>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Literal<'src> {
    Number(&'src str),
    String(String),
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
