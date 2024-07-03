use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{rules::PResult, ParseError, TokenStream},
    T,
};

pub fn try_parse_type(tokens: TokenStream) -> PResult<Option<S<&str>>> {
    if let &[S(Token::Identifier(type_), span)] = tokens {
        return Ok(Some(S(type_, span)));
    } else if let &[S(T!("("), s1), S(T!(")"), s2)] = tokens {
        return Ok(Some(S("()", (s1.start..s2.end).into())));
    }

    Err(ParseError::InvalidType(
        error_handling::span_of(tokens).unwrap(),
    ))
}
