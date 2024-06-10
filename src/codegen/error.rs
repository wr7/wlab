use std::ops::Deref;

use crate::{
    diagnostic as d,
    error_handling::{Diagnostic, Hint, Spanned as S},
    parser::OpCode,
};

use wutil::Span;

use super::types::Type;

pub fn undefined_variable(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined variable `{}`", name.deref()),
        [Hint::new_error("", name.1)],
    }
}
pub fn undefined_function(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined function `{}`", name.deref()),
        [Hint::new_error("", name.1)],
    }
}
pub fn undefined_type(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined type `{}`", name.deref()),
        [Hint::new_error("", name.1)],
    }
}
pub fn undefined_operator(operator: OpCode, span: Span, type_: &Type) -> Diagnostic {
    d! {
        format!("Operator `{operator}` is not defined for type `{type_}`"),
        [Hint::new_error(format!("Value here is of type `{type_}`"), span)],
    }
}
pub fn unexpected_type(span: Span, expected: &Type, got: &Type) -> Diagnostic {
    d! {
        format!("Unexpected type: expected `{expected}`; got `{got}`"),
        [Hint::new_error(format!("value here of type `{got}`"), span)]
    }
}
pub fn invalid_param_count(span: Span, expected: usize, got: usize) -> Diagnostic {
    d! {
        format!("Incorrect number of parameters: expected {expected}, got {got}"),
        [Hint::new_error("Function called here", span)]
    }
}
pub fn invalid_number(num: S<&str>) -> Diagnostic {
    d! {
        format!("Invalid numberical literal `{}`", num.deref()),
        [Hint::new_error("Literal used here", num.1)]
    }
}
