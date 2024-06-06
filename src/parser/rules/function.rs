use wutil::Span;

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

    for (expr, separator) in TokenSplit::new(tokens, |t| t == &T!(",")) {
        let Some(expr) = try_parse_expr(expr)? else {
            let Some(separator) = separator else {
                break;
            };

            return Err(ParseError::ExpectedExpression(separator.1.span_at()));
        };

        expressions.push(expr);
    }

    Ok(expressions)
}

/// Parses function parameters from the front of a token list. Returns `(arguments, remaining_tokens)`
fn parse_fn_params<'a>(
    tokens: &'a [S<Token<'a>>],
    name_span: Span,
) -> PResult<(Vec<(&'a str, &'a str)>, &'a [S<Token<'a>>])> {
    let mut params = Vec::new();

    let mut nb_tokens = NonBracketedIter::new(tokens);

    let Some(open_paren @ S(T!("("), _)) = nb_tokens.next() else {
        return Err(ParseError::ExpectedToken(
            name_span.span_after(),
            &[T!("(")],
        ));
    };

    let Some(close_paren @ S(T!(")"), _)) = nb_tokens.next() else {
        unreachable!()
    };

    let param_range =
        tokens.elem_offset(open_paren).unwrap() + 1..tokens.elem_offset(close_paren).unwrap();

    for (param, separator) in TokenSplit::new(&tokens[param_range], |t| t == &T!(",")) {
        let Some(param) = parse_fn_param(param)? else {
            let Some(separator) = separator else {
                break; // Ignore trailing comma
            };

            return Err(ParseError::ExpectedParameter(separator.1.span_at()));
        };

        params.push(param)
    }

    Ok((params, &tokens[tokens.elem_offset(close_paren).unwrap()..]))
}

/// Parses a function parameter (eg `foo: u32`)
fn parse_fn_param<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<(&'a str, &'a str)>> {
    let Some((S(Token::Identifier(name), name_span), tokens)) = tokens.split_first() else {
        let Some(tok) = tokens.first() else {
            return Ok(None);
        };

        return Err(ParseError::ExpectedParamName(tok.1));
    };

    let Some((S(T!(":"), colon_span), tokens)) = tokens.split_first() else {
        let span = tokens
            .first()
            .map(|t| t.1)
            .unwrap_or(name_span.span_after());

        return Err(ParseError::ExpectedToken(span, &[T!(":")]));
    };

    let Some((S(Token::Identifier(type_), _), tokens)) = tokens.split_first() else {
        let span = tokens
            .first()
            .map(|t| t.1)
            .unwrap_or(colon_span.span_after());

        return Err(ParseError::ExpectedType(span));
    };

    if let Some(tok) = tokens.first() {
        return Err(ParseError::ExpectedToken(tok.1, &[T!(","), T!(")")]));
    }

    Ok(Some((name, type_)))
}
