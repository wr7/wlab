use crate::{
    diagnostic as d,
    error_handling::{Hint, WLangError},
    util::StrExt,
};

pub enum CodegenError<'a> {
    UndefinedVariable(&'a str),
    UndefinedFunction(&'a str),
    InvalidParameters(&'a str, usize, usize), // TODO: add span of function definition
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
            CodegenError::InvalidParameters(name, expected, got) => d! {
                format!("Invalid number of parameters; expected {expected}, got {got}"),
                [Hint::new_error(
                    "Function called here",
                    code.substr_pos(name).unwrap(),
                )],
            },
        }
    }
}
