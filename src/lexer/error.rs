use wutil::Span;

use crate::{
    diagnostic as d,
    error_handling::{Diagnostic, Hint, Spanned},
};

pub fn invalid_token(tok: Spanned<&str>) -> Diagnostic {
    d! {
        format!("Invalid token `{}`", *tok),
        [Hint::new_error("", tok.1)],
    }
}

pub fn unclosed_string(span: Span) -> Diagnostic {
    d! {
        "Unclosed string",
        [Hint::new_error("string starts here", span)],
    }
}

pub fn unclosed_comment(span: Span) -> Diagnostic {
    d! {
        "Unclosed comment",
        [Hint::new_error("comment starts here", span)],
    }
}

pub fn invalid_escape(seq: Spanned<&str>) -> Diagnostic {
    d! {
        format!("Invalid escape sequence \"\\{}\"", *seq),
        [Hint::new_error("", seq.1)],
    }
}
