use crate::{
    error_handling::Spanned as S,
    lexer::Token,
    parser::{
        rules::try_parse_expr,
        util::{NonBracketedIter, TokenSplit},
        Expression, ParseError, Statement,
    },
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

    let mut nb_iter = NonBracketedIter::new(tokens);

    let left_paren = nb_iter.next();
    let Some(left_paren @ S(T!("("), _)) = left_paren else {
        let span = left_paren.map_or(name_span.span_after(), |t| t.1);

        return Err(ParseError::ExpectedToken(span, &[T!("(")]));
    };

    let Some(right_paren @ S(T!(")"), _)) = nb_iter.next() else {
        unreachable!();
    };

    let left_paren_idx = tokens.elem_offset(left_paren).unwrap();
    let right_paren_idx = tokens.elem_offset(right_paren).unwrap();

    let params = left_paren_idx + 1..right_paren_idx;
    let params = parse_fn_params(&tokens[params])?;

    let Some(Expression::CompoundExpression(body)) =
        try_parse_bracket_expr(&tokens[right_paren_idx + 1..])?
    else {
        return Err(ParseError::ExpectedBody(right_paren.1.span_after()));
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

    Ok(Some(Expression::FunctionCall(fn_name, params)))
}

fn parse_expression_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<S<Expression<'a>>>> {
    let mut expressions = Vec::new();

    for (expr_toks, separator) in TokenSplit::new(tokens, |t| t == &T!(",")) {
        let Some(expr) = try_parse_expr(expr_toks)? else {
            let Some(separator) = separator else {
                break;
            };

            return Err(ParseError::ExpectedExpression(separator.1.span_at()));
        };

        let span = expr_toks.first().unwrap().1.start..expr_toks.last().unwrap().1.end;

        expressions.push(S(expr, span.into()));
    }

    Ok(expressions)
}

/// Parses function parameters eg `foo: i32, bar: usize`.
fn parse_fn_params<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<(&'a str, S<&'a str>)>> {
    let mut params = Vec::new();

    for (param, separator) in TokenSplit::new(tokens, |t| t == &T!(",")) {
        let Some(param) = parse_fn_param(param)? else {
            let Some(separator) = separator else {
                break; // Ignore trailing comma
            };

            return Err(ParseError::ExpectedParameter(separator.1.span_at()));
        };

        params.push(param);
    }

    Ok(params)
}

/// Parses a function parameter (eg `foo: u32`)
fn parse_fn_param<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<(&'a str, S<&'a str>)>> {
    let Some((S(Token::Identifier(name), name_span), tokens)) = tokens.split_first() else {
        let Some(tok) = tokens.first() else {
            return Ok(None);
        };

        return Err(ParseError::ExpectedParamName(tok.1));
    };

    let Some((S(T!(":"), colon_span), tokens)) = tokens.split_first() else {
        let span = tokens.first().map_or(name_span.span_after(), |t| t.1);

        return Err(ParseError::ExpectedToken(span, &[T!(":")]));
    };

    let Some(type_) = super::types::try_parse_type(tokens)? else {
        let span = tokens.first().map_or(colon_span.span_after(), |t| t.1);

        return Err(ParseError::ExpectedType(span));
    };

    Ok(Some((name, type_)))
}
