use crate::{
    error_handling::{self, Spanned as S},
    parser::{
        self,
        ast::{Statement, Struct, StructField},
        macros::match_tokens,
        rules::{self, attributes, PResult},
        util::TokenSplit,
        TokenStream,
    },
    T,
};

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
