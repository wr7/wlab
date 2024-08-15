use std::fmt::Write as _;

use wutil::Span;

use crate::{
    error_handling::{Diagnostic, Hint, Spanned},
    lexer::{BracketType, Token},
};

use crate::diagnostic as d;

/// Checks for mismatched/unmatched brackets in an expression
pub fn check_brackets<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Result<(), Diagnostic> {
    let mut brackets: Vec<Spanned<BracketType>> = Vec::new();

    for stoken in tokens {
        let Spanned(token, span) = stoken;
        let span = *span;

        match token {
            Token::OpenBracket(ty) => {
                brackets.push(Spanned(*ty, span));
            }
            Token::CloseBracket(ty) => {
                let Some(open_type) = brackets.pop() else {
                    return Err(unmatched_bracket(stoken.as_sref()));
                };

                if *ty != *open_type {
                    return Err(mismatched_brackets(open_type.1, span));
                }
            }
            _ => continue,
        }
    }

    if let Some(br) = brackets.first() {
        return Err(unmatched_bracket(Spanned(&Token::OpenBracket(br.0), br.1)));
    }

    Ok(())
}

pub fn invalid_expression(span: Span) -> Diagnostic {
    d! {
        "Invalid expression",
        [Hint::new_error("", span)],
    }
}
pub fn invalid_attribute(span: Span) -> Diagnostic {
    d! {
        "Invalid attribute",
        [Hint::new_error("", span)],
    }
}
pub fn unmatched_bracket(tok: Spanned<&Token>) -> Diagnostic {
    d! {
        format!("Unmatched bracket `{}`", tok.as_str()),
        [Hint::new_error("", tok.1)],
    }
}
pub fn expected_function_or_struct(span: Span) -> Diagnostic {
    d! {
        "Expected function or struct definition",
        [Hint::new_error("", span)],
    }
}
pub fn expected_parameter(span: Span) -> Diagnostic {
    d! {
        "Expected function parameter",
        [Hint::new_error("", span)],
    }
}
pub fn expected_body(span: Span) -> Diagnostic {
    d! {
        "Expected function body",
        [Hint::new_error("", span)],
    }
}
pub fn expected_fields(span: Span) -> Diagnostic {
    d! {
        "Expected struct fields",
        [Hint::new_error("", span)],
    }
}
pub fn expected_expression(span: Span) -> Diagnostic {
    d! {
        "Expected expression",
        [Hint::new_error("", span)],
    }
}
pub fn expected_param_name(span: Span) -> Diagnostic {
    d! {
        "Expected function parameter name",
        [Hint::new_error("expected name here", span)],
    }
}
pub fn expected_type(span: Span) -> Diagnostic {
    d! {
        "Expected type",
        [Hint::new_error("", span)],
    }
}
pub fn expected_identifier(span: Span) -> Diagnostic {
    d! {
        "Expected identifier",
        [Hint::new_error("", span)],
    }
}
pub fn expected_token(span: Span, tokens: &[Token]) -> Diagnostic {
    let mut msg = "expected token".to_owned();
    for (i, tok) in tokens.iter().enumerate() {
        if i != 0 && tokens.len() > 2 {
            msg += ",";
        }
        msg += " ";
        if i == tokens.len() - 1 && tokens.len() > 1 {
            msg += "or ";
        }
        write!(&mut msg, "{tok}").unwrap();
    }

    d! {
        msg,
        [Hint::new_error("", span)],
    }
}
pub fn mismatched_brackets(opening: Span, closing: Span) -> Diagnostic {
    d! {
        "Mismatched brackets",
        [
            Hint::new_error("opening bracket here", opening),
            Hint::new_error("closing bracket here", closing),
        ],
    }
}
pub fn unexpected_tokens(span: Span) -> Diagnostic {
    d! {
        "Unexpected tokens",
        [
            Hint::new_error("tokens here", span)
        ]
    }
}
pub fn missing_block(span: Span) -> Diagnostic {
    d! {
        "Missing if statement block",
        [
            Hint::new_error("if statement here", span)
        ]
    }
}
