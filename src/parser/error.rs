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
    InvalidType(Span),
    InvalidAttribute(Span),
    UnmatchedBracket(Span),
    ExpectedFunction(Span),
    ExpectedParameter(Span),
    ExpectedBody(Span),
    ExpectedExpression(Span),
    ExpectedParamName(Span),
    ExpectedType(Span),
    ExpectedIdentifier(Span),
    ExpectedToken(Span, &'static [Token<'static>]),
    UnexpectedTokens(Span),
    MismatchedBrackets(Span, Span),
    MissingBlock(Span),
}

impl WLangError for ParseError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        let mut diagnostic = match self {
            ParseError::InvalidExpression(span) => d! {
                "invalid expression",
                [Hint::new_error("", *span)],
            },
            ParseError::InvalidType(span) => d! {
                "invalid type",
                [Hint::new_error("", *span)],
            },
            ParseError::InvalidAttribute(span) => d! {
                "invalid attribute",
                [Hint::new_error("", *span)],
            },
            ParseError::UnmatchedBracket(span) => d! {
                format!("unmatched bracket `{}`", &code[*span]),
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedFunction(span) => d! {
                "expected function definition",
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
            ParseError::ExpectedParamName(span) => d! {
                format!("expected function parameter name, got `{}`", &code[*span]),
                [Hint::new_error("expected name here", *span)],
            },
            ParseError::ExpectedType(span) => d! {
                format!("expected type, got `{}`", &code[*span]),
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedIdentifier(span) => d! {
                "expected identifier",
                [Hint::new_error("", *span)],
            },
            ParseError::ExpectedToken(span, tokens) => {
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
                    [Hint::new_error("", *span)],
                }
            }
            ParseError::MismatchedBrackets(opening, closing) => d! {
                "mismatched brackets",
                [
                    Hint::new_error("opening bracket here", *opening),
                    Hint::new_error("closing bracket here", *closing),
                ],
            },
            ParseError::UnexpectedTokens(span) => d! {
                "unexpected tokens",
                [
                    Hint::new_error("tokens here", *span)
                ]
            },
            ParseError::MissingBlock(span) => d! {
                "missing if statement block",
                [
                    Hint::new_error("if statement here", *span)
                ]
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
        let span = *span;

        match token {
            Token::OpenBracket(ty) => {
                brackets.push(Spanned(*ty, span));
            }
            Token::CloseBracket(ty) => {
                let Some(open_type) = brackets.pop() else {
                    return Err(ParseError::UnmatchedBracket(span));
                };

                if *ty != *open_type {
                    return Err(ParseError::MismatchedBrackets(open_type.1, span));
                }
            }
            _ => continue,
        }
    }

    if let Some(br) = brackets.first() {
        return Err(ParseError::UnmatchedBracket(br.1));
    }

    Ok(())
}
