use std::ops::Deref;

use wutil::{iter::IterCloneExt, Span};

use crate::{
    error_handling::Spanned as S,
    lexer::Token,
    parser::{rules::try_parse_expr, util::NonBracketedIter, Expression, ParseError, Statement},
    util::SliceExt,
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

    let (params, tokens) = parse_fn_params(tokens, *name_span)?;

    let Some((S(T!(")"), right_paren), tokens)) = tokens.split_first() else {
        unreachable!()
    };

    let Some(Expression::CompoundExpression(body)) = try_parse_bracket_expr(tokens)? else {
        return Err(ParseError::ExpectedBody(right_paren.span_after()));
    };

    Ok(Some(Statement::Function(fn_name, params, body)))
}

pub fn try_parse_function_call<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    let Some(([S(Token::Identifier(fn_name), _), S(T!("("), _)], tokens)) =
        tokens.split_first_chunk()
    else {
        return Ok(None);
    };

    let Some((S(T!(")"), _), tokens)) = tokens.split_last() else {
        return Ok(None); // Will yield an invalidexpression error eventually
    };

    let params = parse_expression_list(tokens)?;

    Ok(Some(Expression::FunctionCall(&fn_name, params)))
}

fn parse_expression_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<Expression<'a>>> {
    let mut expressions = Vec::new();

    // For reporting errors
    let Some(mut last_good_span) = tokens.first().map(|t| t.1) else {
        return Ok(expressions);
    };

    for expr in NonBracketedIter::new(tokens).split(|t| matches!(&t.0, &T!(","))) {
        let expr_range: Span = tokens.range_of(expr).unwrap_or(0..0).into();

        let expr = try_parse_expr(&tokens[expr_range])?
            .ok_or(ParseError::ExpectedExpression(last_good_span.span_after()))?;

        if let Some(tok) = tokens.get(expr_range.end) {
            last_good_span = tok.1.span_after();
        }

        expressions.push(expr);
    }

    Ok(expressions)
}

// TODO: use iterator split
fn parse_fn_params<'a>(
    tokens: &'a [S<Token<'a>>],
    name_span: Span,
) -> PResult<(Vec<&'a str>, &'a [S<Token<'a>>])> {
    let mut params = Vec::new();

    let Some((S(T!("("), _), mut tokens)) = tokens.split_first() else {
        return Err(ParseError::ExpectedToken(
            name_span.span_after(),
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