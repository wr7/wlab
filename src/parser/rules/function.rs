use crate::{
    error_handling::{self, span_of, Spanned as S},
    parser::{
        ast::{self, Expression, Function, Statement, Visibility},
        error,
        macros::match_tokens,
        rules::{
            attributes::try_parse_attributes_from_front,
            bracket_expr::try_parse_code_block_from_front,
            path, try_parse_expr,
            types::{self, try_parse_type_from_front},
            PResult,
        },
        util::TokenSplit,
        TokenStream,
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
                    return Err(error::expected_token(name_tok.1.span_after(), &[T!("(")]))
                } @ (left_paren, params, right_paren);
            };

            all(
                token("->") @ arrow;
                expect_(do_(|toks| try_parse_type_from_front(toks)?)) else {
                    return Err(error::expected_type(arrow.1.span_after()))
                };
            ) @ ret_type;

            required(do_(|toks| try_parse_code_block_from_front(toks)?)) else {
                return Err(error::expected_body(ret_type.map_or(right_paren.1, |(_, ret_ty)| ret_ty.1).span_after()));
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
        } => |remaining| {
            if let Some(span) = error_handling::span_of(remaining) {
                return Err(error::unexpected_tokens(span));
            }

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

            return Err(error::expected_expression(separator.1.span_at()));
        };

        let span = error_handling::span_of(expr_toks).unwrap();

        expressions.push(S(expr, span));
    }

    Ok(expressions)
}

/// Parses function parameters eg `foo: i32, bar: usize`.
fn parse_fn_params<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Vec<(S<&'src str>, S<ast::Path<'src>>)>> {
    let mut params = Vec::new();

    for (param, separator) in TokenSplit::new(tokens, |t| t == &T!(",")) {
        let Some(param) = parse_fn_param(param)? else {
            let Some(separator) = separator else {
                break; // Ignore trailing comma
            };

            return Err(error::expected_parameter(separator.1.span_at()));
        };

        params.push(param);
    }

    Ok(params)
}

/// Parses a function parameter (eg `foo: u32`)
fn parse_fn_param<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<(S<&'src str>, S<ast::Path<'src>>)>> {
    match_tokens! {
        tokens: {
            required {
                ident() else {
                    let Some(tok) = tokens.first() else {
                        return Ok(None);
                    };

                    return Err(error::expected_param_name(tok.1));
                } @ (name, name_tok);

                token(":") else {
                    return Err(error::expected_token(name_tok.1.span_after(), &[T!(":")]));
                } @ colon;

                either(
                    do_(|tokens| {
                        types::try_parse_type_from_front(tokens)?
                    });
                    do_(|tokens| {
                        let span = tokens.first().map_or(colon.1.span_after(), |t| t.1);

                        return Err(error::expected_type(span));
                    });
                ) @ type_;
            };

        } => |tokens| {
            if let Some(tok) = tokens.first() {
                return Err(error::expected_token(tok.1, &[T!(","), T!(")")]))
            }

            Ok(Some((S(name, name_tok.1), type_)))
        }
    }
}
