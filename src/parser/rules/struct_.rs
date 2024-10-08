use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{
        self,
        ast::{Expression, Statement, Struct, StructField, StructInitializerField},
        error,
        macros::match_tokens,
        rules::{self, attributes, path::try_parse_path_from_front, PResult},
        util::TokenSplit,
        TokenStream,
    },
    T,
};

pub fn try_parse_struct_initializer<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<Expression<'src>>> {
    match_tokens! {
        tokens: {
            required {
                do_(|tokens| try_parse_path_from_front(tokens)?) @ name;
                bracketed(
                    BracketType::Curly: {
                        do_(|tokens| parse_struct_initializer_fields(tokens)?);
                    }
                ) @ (_, fields, _);
            };
        } => |remaining_tokens| {
            if let Some(span) = error_handling::span_of(remaining_tokens) {
                return Err(error::unexpected_tokens(span));
            }

            Ok(Some(Expression::StructInitializer { name, fields }))
        }
    }
}

pub fn try_parse_field_access<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<Expression<'src>>> {
    let [tokens @ .., S(T!("."), dot_span), S(Token::Identifier(field_name), field_span)] = tokens
    else {
        return Ok(None);
    };

    let expr = S(
        rules::try_parse_expr(tokens)?
            .ok_or_else(|| parser::error::expected_expression(dot_span.span_at()))?,
        error_handling::span_of(tokens).unwrap(),
    );

    Ok(Some(Expression::FieldAccess(
        Box::new(expr),
        S(field_name, *field_span),
    )))
}

pub fn try_parse_struct_from_front<'a, 'src>(
    tokens: &'a TokenStream<'src>,
) -> PResult<Option<(Statement<'src>, &'a TokenStream<'src>)>> {
    match_tokens! {
        tokens: {
            do_(|tokens| {
                attributes::try_parse_attributes_from_front(tokens)?
            }) @ attributes;

            required {
                token("struct");
                ident() @ (name, name_tok);

                bracketed(BracketType::Curly: {
                    do_(|tokens| parse_struct_fields(tokens)?);
                }) else {
                    return Err(parser::error::expected_fields(name_tok.1.span_after()));
                } @ (_, fields, _)
            }
        } => |remaining| {
            Ok(Some((Statement::Struct(Struct {name, fields, attributes: attributes.unwrap_or_default()}), remaining)))
        }
    }
}

fn parse_struct_initializer_fields<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Vec<S<StructInitializerField<'src>>>> {
    let mut fields = Vec::new();

    for (field_toks, separator) in TokenSplit::new(tokens, |t| t == &T!(",")) {
        let Some(field) = parse_struct_initializer_field(field_toks)? else {
            let Some(separator) = separator else {
                break;
            };

            return Err(parser::error::expected_identifier(separator.1));
        };

        fields.push(S(field, error_handling::span_of(field_toks).unwrap()));
    }

    Ok(fields)
}

fn parse_struct_fields<'src>(tokens: &TokenStream<'src>) -> PResult<Vec<S<StructField<'src>>>> {
    let mut fields = Vec::new();

    for (field, separator) in TokenSplit::new(tokens, |t| t == &T!(",")) {
        let Some(field) = parse_struct_field(field)? else {
            let Some(separator) = separator else {
                break;
            };

            return Err(parser::error::expected_identifier(separator.1));
        };

        fields.push(field);
    }

    Ok(fields)
}

fn parse_struct_initializer_field<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<StructInitializerField<'src>>> {
    match_tokens! {
        tokens: {
            required {
                ident() else {
                    let Some(first_tok) = tokens.first() else {
                        return Ok(None);
                    };
                    return Err(parser::error::expected_identifier(first_tok.1))
                } @ (name, name_tok);

                token(":") else {
                    return Err(parser::error::expected_token(name_tok.1.span_after(), &[T!(":")]))
                } @ colon;
            };

            required(do_(|tokens| {
                rules::try_parse_expr(tokens)?
                    .map(|expr| S(
                        expr,
                        error_handling::span_of(tokens).unwrap()
                    ))
            })) else {
                return Err(parser::error::expected_type(colon.1.span_after()))
            } @ expr;
        } => {
            Ok(Some(StructInitializerField{name: S(name, name_tok.1), val: expr}))
        }
    }
}

fn parse_struct_field<'src>(tokens: &TokenStream<'src>) -> PResult<Option<S<StructField<'src>>>> {
    match_tokens! {
        tokens: {
            required {
                ident() else {
                    let Some(first_tok) = tokens.first() else {
                        return Ok(None);
                    };
                    return Err(parser::error::expected_identifier(first_tok.1))
                } @ (name, name_tok);

                token(":") else {
                    return Err(parser::error::expected_token(name_tok.1.span_after(), &[T!(":")]))
                } @ colon;
            };

            required(do_(|tokens| {
                rules::types::try_parse_type_from_front(tokens)?
            })) else {
                return Err(parser::error::expected_type(colon.1.span_after()))
            } @ type_;
        } => |remaining| {
            if let Some(span) = error_handling::span_of(remaining) {
                return Err(parser::error::unexpected_tokens(span));
            }

            Ok(Some(S(StructField {name, type_}, error_handling::span_of(tokens).unwrap())))
        }
    }
}
