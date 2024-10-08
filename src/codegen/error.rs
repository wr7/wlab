use crate::{
    codegen::types::Type,
    diagnostic as d,
    error_handling::{self, Diagnostic, Hint, Spanned as S},
    parser::ast::{Attribute, CodeBlock, OpCode, Path},
    util,
};

use wutil::Span;

pub fn undefined_variable(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined variable `{}`", &*name),
        [Hint::new_error("", name.1)],
    }
}
pub fn modified_immutable_variable(def_name: S<&str>, mutate_span: Span) -> Diagnostic {
    d! {
        format!("Cannot modify immutable variable `{}`", &*def_name),
        [
            Hint::new_info(format!("Variable declared here; try replacing `{0}` with `mut {0}`", *def_name), def_name.1),
            Hint::new_error("Variable modified here", mutate_span),
        ],
    }
}
pub fn modify_rvalue(rvalue: Span) -> Diagnostic {
    d! {
        "Cannot modify rvalue; try storing it in a mutable variable first",
        [
            Hint::new_error("", rvalue),
        ],
    }
}
pub fn undefined_item(name: S<&str>) -> Diagnostic {
    d! {
        format!("Undefined item `{}`", &*name),
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
pub fn unexpected_break_type(expected: S<&Type>, got: S<&Type>) -> Diagnostic {
    d! {
        format!("Unexpected type: expected `{}`; got `{}`", expected.0, got.0),
        [
            Hint::new_info(format!("expected `{}` because of this code here", expected.0), expected.1),
            Hint::new_error(format!("value here is of type `{}`", got.0), got.1),
        ]
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
        "If and else have different types",
        [
            Hint::new_error(format!("If block is of type `{}`", *if_block), if_block.1),
            Hint::new_error(format!("Else block is of type `{}`", *else_block), else_block.1),
        ]
    }
}

pub fn non_function_attribute(attr: &S<Attribute>) -> Diagnostic {
    d! {
        "Invalid function attribute",
        [
            Hint::new_error("", attr.1),
        ]
    }
}

pub fn non_struct_attribute(attr: &S<Attribute>) -> Diagnostic {
    d! {
        "Invalid struct attribute",
        [
            Hint::new_error("", attr.1),
        ]
    }
}

pub fn non_module_attribute(attr: &S<Attribute>) -> Diagnostic {
    d! {
        "Invalid module attribute",
        [
            Hint::new_error("", attr.1),
        ]
    }
}

pub fn multiple_intrinsic_attributes(first_intrinsic: Span, second_intrinsic: Span) -> Diagnostic {
    d! {
        "Multiple intrinsic attributes on function",
        [
            Hint::new_error("First intrinsic here", first_intrinsic),
            Hint::new_error("Second intrinsic here", second_intrinsic),
        ]
    }
}

pub fn missing_crate_name() -> Diagnostic {
    d! {
        "No crate name declared",
        [
            Hint::new_error("Try #![declare_crate(crate_name)]", Span::at(0)),
        ]
    }
}

pub fn not_module(lhs: S<&str>) -> Diagnostic {
    d! {
        format!("`::` syntax can only be used with types and modules; `{}` is not a module", *lhs),
        [
            Hint::new_error("Non-module item here", lhs.1)
        ]
    }
}

pub fn no_item(parent: Option<&str>, item: S<&str>) -> Diagnostic {
    if let Some(parent) = parent {
        d! {
            format!("No item named `{}` in `{parent}`", *item),
            [
                Hint::new_error("", item.1)
            ]
        }
    } else {
        d! {
            format!("No crate/item named `{}`", *item),
            [
                Hint::new_error("", item.1)
            ]
        }
    }
}

pub fn not_function(name: S<&str>) -> Diagnostic {
    d! {
        format!("Cannot call non-function item `{}`", *name),
        [
            Hint::new_error("", name.1)
        ]
    }
}

pub fn not_type(name: S<&str>) -> Diagnostic {
    d! {
        format!("`{}` is not a type", *name),
        [
            Hint::new_error("", name.1)
        ]
    }
}

pub fn not_function_path(path: &S<Path>) -> Diagnostic {
    let name: String = util::Intersperse::new(path.iter().map(|n| **n), "::").collect();

    not_function(S(&name, path.1))
}

pub fn item_already_defined(item: S<&str>) -> Diagnostic {
    d! {
        format!("An item named {} already exists", *item),
        [
            Hint::new_error("", item.1)
        ]
    }
}

pub fn non_empty_intrinsic(body: Span) -> Diagnostic {
    d! {
        "Intrinsic function body is not empty",
        [
            Hint::new_error("this should be empty `{}`", body)
        ]
    }
}

pub fn invalid_intrinsic(intrinsic: S<&str>) -> Diagnostic {
    d! {
        format!("Invalid intrinsic `{}`", *intrinsic),
        [
            Hint::new_error("", intrinsic.1)
        ]
    }
}

pub fn invalid_intrinsic_params(params_span: Span, expected_params: &str) -> Diagnostic {
    d! {
        format!("Invalid intrinsic parameters; Expected parameters `{}`", expected_params),
        [
            Hint::new_error("", params_span)
        ]
    }
}

pub fn invalid_intrinsic_ret_type(function_span: Span, expected_ret_type: &Type) -> Diagnostic {
    d! {
        format!("Invalid intrinsic return type; Expected type `{}`", expected_ret_type),
        [
            Hint::new_error("", function_span)
        ]
    }
}

pub fn private_function(crate_name: S<&str>, fn_name: S<&str>) -> Diagnostic {
    d! {
        format!("Cannot access private item `{}::{}`", *crate_name, *fn_name),
        [
            Hint::new_error("", error_handling::span_of(&[crate_name, fn_name]).unwrap())
        ]
    }
}

pub fn non_struct_element_access(span: Span, type_: &Type, field: &str) -> Diagnostic {
    d! {
        format!("Cannot access field `{field}` of non-struct type `{type_}`"),
        [ Hint::new_error(format!("expression is of type `{type_}`"), span) ]
    }
}

pub fn non_struct_type_initializer(type_: S<&Type>) -> Diagnostic {
    d! {
        format!("Cannot create struct of type `{0}` because `{0}` is not a struct", *type_),
        [ Hint::new_error("", type_.1) ]
    }
}

pub fn invalid_field(path: &str, field: S<&str>) -> Diagnostic {
    d! {
        format!("No field `{}` in struct `{path}`", *field),
        [ Hint::new_error("", field.1) ]
    }
}

pub fn duplicate_field(field1: S<&str>, field2: Span) -> Diagnostic {
    d! {
        format!("Field `{}` is defined multiple times", field1.0),
        [
            Hint::new_info("first defined here", field1.1),
            Hint::new_error("then defined here", field2),
        ]
    }
}

pub fn missing_field(field: &str, struct_name: S<&str>) -> Diagnostic {
    d! {
        format!("Missing field `{field}` in struct {}", *struct_name),
        [
            Hint::new_error("", struct_name.1),
        ]
    }
}

pub fn duplicate_main(other_main: &str, this_crate: &str, span: Span) -> Diagnostic {
    d! {
        format!("`main` function defined twice: first defined in `{other_main}` then in `{this_crate}`"),
        [ Hint::new_error("", span) ]
    }
}

pub fn main_arguments(span: Span) -> Diagnostic {
    d! {
        "main function cannot have arguments",
        [ Hint::new_error("", span) ]
    }
}

pub fn no_exit(crate_name: &str) -> Diagnostic {
    d! {
        format!("`std::exit(i32)` does not exist; this is required for `{crate_name}::main`"),
        []
    }
}

pub fn exit_arguments() -> Diagnostic {
    d! {
        "if a `main` function exists, `std::exit` must be have parameters `(i32)`",
        []
    }
}

pub fn main_return_type(span: Span) -> Diagnostic {
    d! {
        "main function must return `()` (unit) type",
        [ Hint::new_error("", span) ]
    }
}

pub fn break_outside_of_loop(span: Span) -> Diagnostic {
    d! {
        "Break statements can only be used within loops. Did you mean `return` instead?",
        [ Hint::new_error("", span) ]
    }
}
