use std::fmt::Display;

use crate::{error_handling::Spanned as S, lexer::Token};

mod error;
mod rules;
mod util;

pub use error::ParseError;
use wutil::Span;

pub type TokenStream<'a> = &'a [S<Token<'a>>];
pub type Path<'a> = Vec<S<&'a str>>;

pub fn parse_module<'a>(mut tokens: &'a [S<Token<'a>>]) -> Result<Module<'a>, ParseError> {
    error::check_brackets(tokens)?;

    let attributes;
    if let Some((attributes_, tokens_)) = rules::try_parse_outer_attributes_from_front(tokens)? {
        tokens = tokens_;
        attributes = attributes_;
    } else {
        attributes = Vec::new();
    }

    let statements = rules::parse_statement_list(tokens)?;
    let functions: Result<Vec<S<Function<'a>>>, _> = statements
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
pub struct Module<'a> {
    pub attributes: Vec<S<Attribute>>,
    pub functions: Vec<S<Function<'a>>>,
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
pub struct Function<'a> {
    pub name: &'a str,
    pub params: Vec<(&'a str, S<&'a str>)>,
    pub return_type: Option<S<&'a str>>,
    pub attributes: Vec<S<Attribute>>,
    pub visibility: Visibility,
    pub body: S<CodeBlock<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Let(&'a str, Box<S<Expression<'a>>>),
    Assign(&'a str, Box<S<Expression<'a>>>),
    Function(Function<'a>),
}

impl<'a> TryFrom<Statement<'a>> for Function<'a> {
    type Error = ();

    fn try_from(stmnt: Statement<'a>) -> Result<Self, Self::Error> {
        if let Statement::Function(f) = stmnt {
            Ok(f)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Identifier(&'a str),
    Literal(Literal<'a>),
    BinaryOperator(Box<S<Self>>, OpCode, Box<S<Self>>),
    CompoundExpression(CodeBlock<'a>),
    FunctionCall(S<Path<'a>>, Vec<S<Expression<'a>>>),
    If {
        condition: Box<S<Self>>,
        block: S<CodeBlock<'a>>,
        else_block: Option<S<CodeBlock<'a>>>,
    },
}

impl<'a> From<Expression<'a>> for Statement<'a> {
    fn from(expr: Expression<'a>) -> Self {
        Statement::Expression(expr)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CodeBlock<'a> {
    pub body: Vec<S<Statement<'a>>>,
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
