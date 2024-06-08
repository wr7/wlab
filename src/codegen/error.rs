use std::borrow::Cow;

use crate::{
    diagnostic as d,
    error_handling::{Hint, WLangError},
    parser::OpCode,
};

use wutil::{prelude::*, Span};

pub enum CodegenError<'a> {
    UndefinedVariable(&'a str),
    UndefinedFunction(&'a str),
    UndefinedType(&'a str),
    UndefinedOperator(OpCode, Span, String),
    UnexpectedType(Span, Cow<'static, str>, String),
    InvalidParamCount(Span, usize, usize),
    InvalidNumber(&'a str),
}

impl<'a> WLangError for CodegenError<'a> {
    fn get_diagnostic(&self, code: &str) -> crate::error_handling::Diagnostic {
        match self {
            CodegenError::UndefinedVariable(name) => d! {
                format!("Undefined variable `{name}`"),
                [Hint::new_error("", code.substr_pos(name).unwrap())],
            },
            CodegenError::UndefinedFunction(name) => d! {
                format!("Undefined function `{name}`"),
                [Hint::new_error("", code.substr_pos(name).unwrap())],
            },
            CodegenError::UndefinedType(name) => d! {
                format!("Undefined type `{name}`"),
                [Hint::new_error("", code.substr_pos(name).unwrap())],
            },
            CodegenError::UndefinedOperator(operator, span, type_) => d! {
                format!("Operator `{operator}` is not defined for type `{type_}`"),
                [Hint::new_error(format!("Value here is of type `{type_}`"), *span)],
            },
            CodegenError::UnexpectedType(span, expected, got) => {
                d! {
                    format!("Unexpected type: expected `{expected}`; got `{got}`"),
                    [Hint::new_error(format!("value here of type `{got}`"), *span)]
                }
            }
            CodegenError::InvalidParamCount(span, expected, got) => {
                d! {
                    format!("Incorrect number of parameters: expected {expected}, got {got}"),
                    [Hint::new_error("Function called here", *span)]
                }
            }
            CodegenError::InvalidNumber(num) => d! {
                format!("Invalid numberical literal `{num}`"),
                [Hint::new_error("Literal used here", code.substr_pos(num).unwrap())]
            },
        }
    }
}
