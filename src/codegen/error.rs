use crate::{
    error_handling::{Diagnostic, Hint, WLangError},
    util::{self, StrExt},
};

pub enum CodegenError<'a> {
    UndefinedVariable(&'a str),
}

impl<'a> WLangError for CodegenError<'a> {
    fn get_diagnostic(&self, code: &str) -> crate::error_handling::Diagnostic {
        match self {
            CodegenError::UndefinedVariable(varname) => Diagnostic {
                msg: format!("Undefined variable `{varname}`").into(),
                hints: vec![Hint::new_error("", code.substr_pos(varname).unwrap())],
            },
        }
    }
}
