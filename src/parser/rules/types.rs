use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::ParseError,
    T,
};

use super::PResult;

pub fn try_parse_type<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<S<&'a str>>> {
    if let &[S(Token::Identifier(type_), span)] = tokens {
        return Ok(Some(S(type_, span)));
    } else if let &[S(T!("("), s1), S(T!(")"), s2)] = tokens {
        return Ok(Some(S("()", (s1.start..s2.end).into())));
    }

    Err(ParseError::InvalidType(
        error_handling::span_of(tokens).unwrap(),
    ))
}
