use crate::{error_handling::Spanned as S, lexer::Token, parser::ParseError, T};

use super::PResult;

pub fn try_parse_type<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<S<&'a str>>> {
    if let &[S(Token::Identifier(type_), span)] = tokens {
        return Ok(Some(S(type_, span)));
    } else if let &[S(T!("("), s1), S(T!(")"), s2)] = tokens {
        return Ok(Some(S("()", (s1.start..s2.end).into())));
    }

    let Some((first_tok, last_tok)) = tokens.first().zip(tokens.last()) else {
        return Ok(None);
    };

    Err(ParseError::InvalidType(
        (first_tok.1.start..last_tok.1.end).into(),
    ))
}
