use crate::{
    diagnostic as d,
    error_handling::{Diagnostic, Hint, Spanned as S},
    parser::{CodeBlock, OpCode},
};

use wutil::Span;

use super::types::Type;

pub fn undefined_variable(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined variable `{}`", &*name),
        [Hint::new_error("", name.1)],
    }
}
pub fn undefined_function(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined function `{}`", &*name),
        [Hint::new_error("", name.1)],
    }
}
pub fn undefined_type(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined type `{}`", &*name),
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
        format!("Invalid numberical literal `{}`", &*num),
        [Hint::new_error("Literal used here", num.1)]
    }
}
pub fn incorrect_return_type(body: S<&CodeBlock>, expected: &Type, got: &Type) -> Diagnostic {
    if body.body.is_empty() {
        return d! {
            format!("Expected return type `{expected}` from function body"),
            [Hint::new_error("Function body is empty", body.1)]
        };
    }

    if expected == &Type::unit {
        return d! {
            format!("Incorrect return type; expected `()`, got `{got}`"),
            [Hint::new_error("Try adding a semicolon here", body.body.last().unwrap().1.span_after())]
        };
    }

    if let Some(semicolon) = body.trailing_semicolon {
        return d! {
            format!("Incorrect return type; expected `{expected}`, got `()`"),
            [Hint::new_error("`()` explicitly returned because of this semicolon here", semicolon)]
        };
    }

    d! {
        format!("Incorrect return type; expected `{expected}`, got `{got}`"),
        [Hint::new_error(format!("Expression here is of type `{got}`"), body.body.last().unwrap().1)]
    }
}
pub fn mismatched_if_else(if_block: S<&Type>, else_block: S<&Type>) -> Diagnostic {
    d! {
        format!("If and else have different types"),
        [
            Hint::new_error(format!("If block is of type `{}`", *if_block), if_block.1),
            Hint::new_error(format!("Else block is of type `{}`", *else_block), else_block.1),
        ]
    }
}
