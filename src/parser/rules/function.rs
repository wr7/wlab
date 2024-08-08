use crate::{
    error_handling::{self, span_of, Spanned as S},
    lexer::Token,
    parser::{
        ast::{Expression, Function, Statement, Visibility},
        macros::match_tokens,
        rules::{
            attributes::try_parse_attributes_from_front,
            bracket_expr::try_parse_code_block_from_front, path, try_parse_expr,
            types::try_parse_type_from_front, PResult,
        },
        util::TokenSplit,
        ParseError, TokenStream,
    },
    T,
};

/// A function. Eg `fn foo() {let x = ten; x}`
pub fn try_parse_function_from_front<'a, 'src>(
    tokens: &'a TokenStream<'src>,
) -> PResult<Option<(Statement<'src>, &'a TokenStream<'src>)>> {
    match_tokens! {
        tokens: {
            do_(|toks| try_parse_attributes_from_front(toks)?) @ attributes;
            token("pub") @ visibility;

            required {
                token("fn");
                ident() @ (name, name_tok);

                bracketed(BracketType::Parenthesis: {
                    do_(|toks| parse_fn_params(toks)?)
                }) else {
                    return Err(ParseError::ExpectedToken(name_tok.1.span_after(), &[T!("(")]))
                } @ (left_paren, params, right_paren);
            };

            all(
                token("->") @ arrow;
                expect_(do_(|toks| try_parse_type_from_front(toks)?)) else {
                    return Err(ParseError::ExpectedType(arrow.1.span_after()))
                };
            ) @ ret_type;

            required(do_(|toks| try_parse_code_block_from_front(toks)?)) else {
                return Err(ParseError::ExpectedBody(ret_type.map_or(right_paren.1, |(_, ret_ty)| ret_ty.1).span_after()));
            } @ (body, remaining);
        } => {
            let visibility = if visibility.is_some() {
                Visibility::Public
            } else {
                Visibility::Private
            };

            Ok(Some((
                Statement::Function(
                    Function {
                        name,
                        params: S(params, span_of(&[left_paren.as_sref(), right_paren.as_sref()]).unwrap()),
                        return_type: ret_type.map(|(_, t)| t),
                        attributes: attributes.unwrap_or_default(),
                        visibility,
                        body,
                    }
                ),
                remaining
            )))
        }
    }
}

pub fn try_parse_function_call<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<Expression<'src>>> {
    match_tokens!(
        tokens: {
            required {
                do_(|tokens| path::try_parse_path_from_front(tokens)?) @ path;
                bracketed(
                    BracketType::Parenthesis: {
                        do_(|tokens| parse_expression_list(tokens)?)
                    }
                ) @ (_, params, _);
            };
        } => {
            Ok(Some(Expression::FunctionCall(path, params)))
        }
    )
}

fn parse_expression_list<'src>(tokens: &TokenStream<'src>) -> PResult<Vec<S<Expression<'src>>>> {
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
fn parse_fn_params<'src>(tokens: &TokenStream<'src>) -> PResult<Vec<(&'src str, S<&'src str>)>> {
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
fn parse_fn_param<'src>(tokens: &TokenStream<'src>) -> PResult<Option<(&'src str, S<&'src str>)>> {
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
