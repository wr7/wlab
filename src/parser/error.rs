use std::fmt::Write as _;

use wutil::Span;

use crate::{
    error_handling::{Diagnostic, Hint, Spanned, WLangError},
    lexer::{BracketType, Token},
};

use crate::diagnostic as d;

#[derive(Debug)]
pub enum ParseError {
    InvalidExpression(Span),
    InvalidParameter(Span),
    UnmatchedBracket(Span),
    ExpectedParameter(Span),
    ExpectedBody(Span),
    ExpectedExpression(Span),
    ExpectedToken(Span, &'static [Token<'static>]),
    MismatchedBrackets(Span, Span),
}

impl WLangError for ParseError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        let mut diagnostic = match self {
            ParseError::InvalidExpression(span) => d! {
                "invalid expression",
                [Hint::new_error("", *span)],
            },
            ParseError::InvalidParameter(span) => d! {
                "invalid parameter",
                [Hint::new_error("", *span)],
            },
            ParseError::UnmatchedBracket(span) => d! {
                format!("unmatched bracket `{}`", &code[*span]),
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedParameter(span) => d! {
                "expected function parameter",
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedBody(span) => d! {
                "expected function body",
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedExpression(span) => d! {
                "expected expression",
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedToken(span, tokens) => {
                let mut msg = "expected token".to_owned();
                for (i, tok) in tokens.iter().enumerate() {
                    if i != 0 && tokens.len() > 2 {
                        msg += ","
                    }
                    msg += " ";
                    if i == tokens.len() - 1 {
                        msg += "or "
                    }
                    write!(&mut msg, "{tok}").unwrap();
                }

                d! {
                    msg,
                    [Hint::new_error("", span.clone())],
                }
            }
            ParseError::MismatchedBrackets(opening, closing) => d! {
                "mismatched brackets",
                [
                    Hint::new_error("opening bracket here", opening.clone()),
                    Hint::new_error("closing bracket here", closing.clone()),
                ],
            },
        };

        diagnostic.msg = format!("Error while parsing code: {}", &diagnostic.msg).into();

        diagnostic
    }
}

/// Checks for mismatched/unmatched brackets in an expression
pub fn check_brackets<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Result<(), ParseError> {
    let mut brackets: Vec<Spanned<BracketType>> = Vec::new();

    for token in tokens {
        let Spanned(token, span) = token;
        let span = span.clone();

        match token {
            Token::OpenBracket(ty) => {
                brackets.push(Spanned(*ty, span));
            }
            Token::CloseBracket(ty) => {
                let Some(open_type) = brackets.pop() else {
                    return Err(ParseError::UnmatchedBracket(span));
                };

                if *ty != open_type.0 {
                    return Err(ParseError::MismatchedBrackets(open_type.1, span));
                }
            }
            _ => continue,
        }
    }

    if let Some(br) = brackets.first() {
        return Err(ParseError::UnmatchedBracket(br.1.clone()));
    }

    Ok(())
}
