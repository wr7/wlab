use std::ops::Range;

use crate::{
    error_handling::{Diagnostic, Hint, Spanned, WLangError},
    lexer::{BracketType, Token},
};

type Span = Range<usize>;

#[derive(Debug)]
pub enum ParseError {
    InvalidExpression(Span),
    UnmatchedBracket(Span),
    ExpectedParameters(Span),
    ExpectedBody(Span),
    ExpectedExpression(Span),
    ExpectedToken(Span, Token<'static>),
    MismatchedBrackets(Span, Span),
}

impl WLangError for ParseError {
    fn get_diagnostic(&self, code: &str) -> Diagnostic {
        let mut diagnostic = match self {
            ParseError::InvalidExpression(span) => Diagnostic {
                msg: "invalid expression".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::UnmatchedBracket(span) => Diagnostic {
                msg: format!("unmatched bracket `{}`", &code[span.clone()]).into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedParameters(span) => Diagnostic {
                msg: "expected function parameters `()`".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedBody(span) => Diagnostic {
                msg: "expected function body".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedExpression(span) => Diagnostic {
                msg: "expected expression".into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::ExpectedToken(span, tok) => Diagnostic {
                msg: format!("expected token {}", tok).into(),
                hints: vec![Hint::new_error("", span.clone())],
            },
            ParseError::MismatchedBrackets(opening, closing) => Diagnostic {
                msg: "mismatched brackets".into(),
                hints: vec![
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
