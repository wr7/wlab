use std::borrow::Cow;

use crate::{
    diagnostic as d,
    error_handling::{Hint, WLangError},
};

use wutil::{prelude::*, Span};

pub enum CodegenError<'a> {
    UndefinedVariable(&'a str),
    UndefinedFunction(&'a str),
    UndefinedType(&'a str),
    InvalidParameters(&'a str, usize, usize), // TODO: add span of function definition
    #[allow(unused)] // TODO: Parser support is needed for Span
    IncorrectType(Span, Cow<'static, str>, String),
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
            CodegenError::InvalidParameters(name, expected, got) => d! {
                format!("Invalid number of parameters; expected {expected}, got {got}"),
                [Hint::new_error(
                    "Function called here",
                    code.substr_pos(name).unwrap(),
                )],
            },
            CodegenError::IncorrectType(span, expected, got) => d! {
                format!("Incorrect type: expected `{expected}`; got `{got}`"),
                [Hint::new_error("Value here", *span)]
            },
            CodegenError::InvalidNumber(num) => d! {
                format!("Invalid numberical literal `{num}`"),
                [Hint::new_error("Literal used here", code.substr_pos(num).unwrap())]
            },
        }
    }
}
