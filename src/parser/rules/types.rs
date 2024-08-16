use wutil::Span;

use crate::{
    error_handling::Spanned as S,
    lexer::Token,
    parser::{self, ast, TokenStream},
    util::MaybeVec,
    T,
};

use super::PResult;

pub fn try_parse_type_from_front<'src>(
    tokens: &mut &TokenStream<'src>,
) -> PResult<Option<S<ast::Path<'src>>>> {
    if let [S(T!("("), s1), S(T!(")"), s2), rem @ ..] = *tokens {
        *tokens = rem;
        let span = (s1.start..s2.end).into();
        Ok(Some(S(MaybeVec::of(S("()", span)), span)))
    } else {
        let Some(start) = tokens.first().map(|t| t.1.start) else {
            return Ok(None);
        };

        let mut end: usize;
        let mut path = MaybeVec::new();

        let mut iter = tokens.iter();
        let mut prev_separator: Option<Span> = None;

        loop {
            let Some(tok) = iter.next() else {
                let Some(prev_separator) = prev_separator else {
                    return Ok(None);
                };

                return Err(parser::error::expected_identifier(
                    prev_separator.span_after(),
                ));
            };

            let Token::Identifier(ident) = **tok else {
                if prev_separator.is_none() {
                    return Ok(None);
                }

                return Err(parser::error::expected_identifier(tok.1));
            };

            path.push(S(ident, tok.1));
            end = tok.1.end;

            let Some(&S(T!("::"), span)) = iter.clone().next() else {
                break;
            };
            iter.next();

            prev_separator = Some(span);
        }

        *tokens = iter.as_slice();

        Ok(Some(S(path, (start..end).into())))
    }
}
