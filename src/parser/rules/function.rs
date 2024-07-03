use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{
        rules::try_parse_expr,
        util::{NonBracketedIter, TokenSplit},
        Expression, Function, ParseError, Statement, Visibility,
    },
    util::SliceExt,
    T,
};

use super::{attributes, bracket_expr::try_parse_code_block_from_front, path, PResult};

/// A function. Eg `fn foo() {let x = ten; x}`
pub fn try_parse_function_from_front<'a>(
    mut tokens: &'a [S<Token<'a>>],
) -> PResult<Option<(Statement<'a>, &'a [S<Token<'a>>])>> {
    let attributes;
    if let Some((attributes_, remaining_tokens)) =
        attributes::try_parse_attributes_from_front(tokens)?
    {
        tokens = remaining_tokens;
        attributes = attributes_;
    } else {
        attributes = Vec::new();
    }

    let visibility;
    if let Some((S(T!("pub"), _), tokens_)) = tokens.split_first() {
        tokens = tokens_;
        visibility = Visibility::Public;
    } else {
        visibility = Visibility::Private;
    }

    let Some(([S(T!("fn"), _), S(Token::Identifier(name), name_span)], tokens)) =
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

    let tokens = &tokens[right_paren_idx + 1..];

    let body_start = tokens
        .iter()
        .position(|t| matches!(&**t, &T!("{")))
        .unwrap_or(tokens.len());

    let return_type =
        if let Some((S(T!("->"), arrow_span), return_type)) = &tokens[..body_start].split_first() {
            let Some(return_type) = super::types::try_parse_type(return_type)? else {
                return Err(ParseError::ExpectedType(arrow_span.span_after()));
            };

            Some(return_type)
        } else {
            None
        };

    let Some((body, remaining_tokens)) = try_parse_code_block_from_front(&tokens[body_start..])?
    else {
        return Err(ParseError::ExpectedBody(right_paren.1.span_after()));
    };

    Ok(Some((
        Statement::Function(Function {
            name,
            params,
            return_type,
            attributes,
            visibility,
            body,
        }),
        remaining_tokens,
    )))
}

pub fn try_parse_function_call<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    let Some((fn_name, remaining_tokens)) = path::try_parse_path_from_front(tokens)? else {
        return Ok(None);
    };

    let tokens = remaining_tokens;
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some(S(T!("("), _)) = nb_iter.next() else {
        return Ok(None);
    };

    let Some(right_paren @ S(T!(")"), _)) = nb_iter.next() else {
        unreachable!()
    };

    let closing_idx = tokens.elem_offset(right_paren).unwrap();

    let trailing_tokens = &tokens[closing_idx + 1..];
    if let Some(span) = error_handling::span_of(trailing_tokens) {
        return Err(ParseError::UnexpectedTokens(span));
    }

    let params = parse_expression_list(&tokens[1..closing_idx])?;

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

        let span = error_handling::span_of(expr_toks).unwrap();

        expressions.push(S(expr, span));
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
