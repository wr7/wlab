use std::fmt::Display;

use wutil::Span;

use crate::{error_handling::Spanned as S, util::MaybeVec};

pub type Path<'src> = MaybeVec<S<&'src str>>;

#[derive(Debug, PartialEq, Eq)]
pub struct Module<'src> {
    pub attributes: Vec<S<Attribute<'src>>>,
    pub functions: Vec<S<Function<'src>>>,
    pub structs: Vec<S<Struct<'src>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Attribute<'src> {
    DeclareCrate(&'src str),
    Intrinsic(&'src str),
    NoMangle,
    Packed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function<'src> {
    pub name: &'src str,
    pub params: S<Vec<(S<&'src str>, S<Path<'src>>)>>,
    pub return_type: Option<S<Path<'src>>>,
    pub attributes: Vec<S<Attribute<'src>>>,
    pub visibility: Visibility,
    pub body: S<CodeBlock<'src>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'src> {
    Expression(Expression<'src>),
    Let {
        name: S<&'src str>,
        value: Box<S<Expression<'src>>>,
        mutable: bool,
    },
    Assign {
        lhs: S<Expression<'src>>,
        rhs: S<Expression<'src>>,
    },
    Function(Function<'src>),
    Struct(Struct<'src>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructField<'src> {
    pub name: &'src str,
    pub type_: S<Path<'src>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructInitializerField<'src> {
    pub name: S<&'src str>,
    pub val: S<Expression<'src>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Struct<'src> {
    pub name: &'src str,
    pub fields: Vec<S<StructField<'src>>>,
    pub attributes: Vec<S<Attribute<'src>>>,
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
    Loop(S<CodeBlock<'src>>),
    StructInitializer {
        name: S<Path<'src>>,
        fields: Vec<S<StructInitializerField<'src>>>,
    },
    FieldAccess(Box<S<Self>>, S<&'src str>),
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
