use std::ops::{Deref, Range};

use crate::{
    error_handling::Spanned as S,
    lexer::Token,
    parser::{Expression, ParseError, Statement},
    T,
};

use super::{bracket_expr::try_parse_bracket_expr, PResult};

/// A function. Eg `fn foo() {let x = ten; x}`
pub fn try_parse_function<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    let Some(([S(T!("fn"), _), S(Token::Identifier(fn_name), name_span)], tokens)) =
        tokens.split_first_chunk::<2>()
    else {
        return Ok(None);
    };

    let (params, tokens) = parse_fn_params(tokens, name_span.clone())?;

    let Some((S(T!(")"), right_paren), tokens)) = tokens.split_first() else {
        unreachable!()
    };

    let Some(Expression::CompoundExpression(body)) = try_parse_bracket_expr(tokens)? else {
        return Err(ParseError::ExpectedBody(right_paren.end..right_paren.end));
    };

    Ok(Some(Statement::Function(&fn_name, params, body)))
}

fn parse_fn_params<'a>(
    tokens: &'a [S<Token<'a>>],
    name_span: Range<usize>,
) -> PResult<(Vec<&'a str>, &'a [S<Token<'a>>])> {
    let mut params = Vec::new();

    let Some((S(T!("("), _), mut tokens)) = tokens.split_first() else {
        return Err(ParseError::ExpectedToken(
            name_span.end..name_span.end,
            &[T!("(")],
        ));
    };

    while let Some((param, tokens_)) = tokens.split_first() {
        if param.deref() == &T!(")") {
            return Ok((params, tokens));
        }

        tokens = tokens_;

        let Token::Identifier(param) = param.deref() else {
            return Err(ParseError::ExpectedIdentifier(param.1.clone()));
        };

        params.push(*param);

        let Some((next_tok, tokens_)) = tokens.split_first() else {
            unreachable!()
        };

        match next_tok.deref() {
            T!(")") => return Ok((params, tokens)),
            T!(",") => (),
            _ => {
                return Err(ParseError::ExpectedToken(
                    next_tok.1.clone(),
                    &[T!(")"), T!(",")],
                ))
            }
        }

        tokens = tokens_;
    }

    unreachable!()
}
