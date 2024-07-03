use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{
        rules::PResult,
        util::{NonBracketedIter, TokenSplit},
        Attribute,
    },
    util::SliceExt as _,
    T,
};

use wutil::iter::IterExt as _;

pub fn try_parse_attributes_from_front<'a>(
    tokens: &'a [S<Token<'a>>],
) -> PResult<Option<(Vec<S<Attribute>>, &'a [S<Token<'a>>])>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some([S(T!("#"), _), S(T!("["), _)]) = nb_iter.collect_n() else {
        return Ok(None);
    };

    let Some(close_bracket) = nb_iter.next() else {
        unreachable!()
    };

    let close_idx = tokens.elem_offset(close_bracket).unwrap();

    let attributes = parse_attribute_list(&tokens[2..close_idx])?;

    Ok(Some((attributes, nb_iter.remainder())))
}

pub fn try_parse_outer_attributes_from_front<'a>(
    tokens: &'a [S<Token<'a>>],
) -> PResult<Option<(Vec<S<Attribute>>, &'a [S<Token<'a>>])>> {
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

fn parse_attribute_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<S<Attribute>>> {
    TokenSplit::new(tokens, |t| t == &T!(","))
        .filter_map(|(toks, _)| parse_attribute(toks))
        .collect()
}

fn parse_attribute<'a>(tokens: &'a [S<Token<'a>>]) -> Option<PResult<S<Attribute>>> {
    Some(Ok(S(
        match *tokens {
            [S(T!("no_mangle"), _)] => Attribute::NoMangle,
            [S(T!("declare_crate"), _), S(T!("("), _), S(Token::Identifier(crate_name), _), S(T!(")"), _)] => {
                Attribute::DeclareCrate(crate_name.into())
            }
            _ => {
                return Some(Err(crate::parser::ParseError::InvalidAttribute(
                    error_handling::span_of(tokens)?,
                )))
            }
        },
        error_handling::span_of(tokens)?,
    )))
}
