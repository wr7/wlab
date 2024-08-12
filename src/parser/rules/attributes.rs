use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{
        ast::Attribute,
        error,
        macros::match_tokens,
        rules::PResult,
        util::{NonBracketedIter, TokenSplit},
        TokenStream,
    },
    util::SliceExt as _,
    T,
};

use wutil::iter::IterExt as _;

pub fn try_parse_attributes_from_front<'src>(
    tokens: &mut &TokenStream<'src>,
) -> PResult<Option<Vec<S<Attribute<'src>>>>> {
    let toks = *tokens;

    match_tokens! {toks: {
        required {
            token("#");
            bracketed(BracketType::Square: {
                do_(|t| parse_attribute_list(t)?);
            }) @ (_, attrs, _);
        };
    } => |remaining| {
        *tokens = remaining;
        Ok(Some(attrs))
    }}
}

pub fn try_parse_outer_attributes_from_front<'a, 'src>(
    tokens: &'a TokenStream<'src>,
) -> PResult<Option<(Vec<S<Attribute<'src>>>, &'a TokenStream<'src>)>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some([S(T!("#"), _), S(T!("!"), _), S(T!("["), _)]) = nb_iter.collect_n() else {
        return Ok(None);
    };

    let Some(close_bracket) = nb_iter.next() else {
        unreachable!()
    };

    let close_idx = tokens.elem_offset(close_bracket).unwrap();

    let attributes = parse_attribute_list(&tokens[3..close_idx])?;

    Ok(Some((attributes, nb_iter.remainder())))
}

fn parse_attribute_list<'src>(tokens: &TokenStream<'src>) -> PResult<Vec<S<Attribute<'src>>>> {
    TokenSplit::new(tokens, |t| t == &T!(","))
        .filter_map(|(toks, _)| parse_attribute(toks))
        .collect()
}

fn parse_attribute<'src>(tokens: &TokenStream<'src>) -> Option<PResult<S<Attribute<'src>>>> {
    Some(Ok(S(
        match *tokens {
            [S(T!("no_mangle"), _)] => Attribute::NoMangle,
            [S(T!("intrinsic"), _), S(T!("("), _), S(Token::Identifier(intrinsic), _), S(T!(")"), _)] => {
                Attribute::Intrinsic(intrinsic)
            }
            [S(T!("declare_crate"), _), S(T!("("), _), S(Token::Identifier(crate_name), _), S(T!(")"), _)] => {
                Attribute::DeclareCrate(crate_name)
            }
            _ => {
                return Some(Err(error::invalid_attribute(error_handling::span_of(
                    tokens,
                )?)))
            }
        },
        error_handling::span_of(tokens)?,
    )))
}
