use crate::{
    error_handling::{Diagnostic, Hint, WLangError},
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
            CodegenError::UndefinedVariable(name) => Diagnostic {
                msg: format!("Undefined variable `{name}`").into(),
                hints: vec![Hint::new_error("", code.substr_pos(name).unwrap())],
            },
            CodegenError::UndefinedFunction(name) => Diagnostic {
                msg: format!("Undefined function `{name}`").into(),
                hints: vec![Hint::new_error("", code.substr_pos(name).unwrap())],
            },
            CodegenError::InvalidParameters(name, expected, got) => Diagnostic {
                msg: format!("Invalid number of parameters; expected {expected}, got {got}").into(),
                hints: vec![Hint::new_error(
                    "Function called here",
                    code.substr_pos(name).unwrap(),
                )],
            },
        }
    }
}
