use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{rules::PResult, ParseError, TokenStream},
    T,
};

pub fn try_parse_type<'src>(tokens: &TokenStream<'src>) -> PResult<Option<S<&'src str>>> {
    if let &[S(Token::Identifier(type_), span)] = tokens {
        return Ok(Some(S(type_, span)));
    } else if let &[S(T!("("), s1), S(T!(")"), s2)] = tokens {
        return Ok(Some(S("()", (s1.start..s2.end).into())));
    }

    Err(ParseError::InvalidType(
        error_handling::span_of(tokens).unwrap(),
    ))
}

pub fn try_parse_type_from_front<'src>(
    tokens: &mut &TokenStream<'src>,
) -> PResult<Option<S<&'src str>>> {
    if let &[S(Token::Identifier(type_), span), ..] = *tokens {
        *tokens = &tokens[1..];
        return Ok(Some(S(type_, span)));
    } else if let &[S(T!("("), s1), S(T!(")"), s2), ..] = *tokens {
        *tokens = &tokens[2..];
        return Ok(Some(S("()", (s1.start..s2.end).into())));
    }

    Err(ParseError::InvalidType(
        error_handling::span_of(tokens).unwrap(),
    ))
}
