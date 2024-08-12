use crate::{error_handling::Spanned as S, lexer::Token, parser::TokenStream, T};

pub fn try_parse_type_from_front<'src>(tokens: &mut &TokenStream<'src>) -> Option<S<&'src str>> {
    match *tokens {
        [S(Token::Identifier(type_), span), rem @ ..] => {
            *tokens = rem;
            Some(S(type_, *span))
        }

        [S(T!("("), s1), S(T!(")"), s2), rem @ ..] => {
            *tokens = rem;
            Some(S("()", (s1.start..s2.end).into()))
        }

        _ => None,
    }
}
