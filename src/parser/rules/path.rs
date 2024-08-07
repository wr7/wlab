use wutil::Span;

use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    parser::{ast::Path, rules::PResult, ParseError, TokenStream},
    util::SliceExt,
    T,
};

pub fn try_parse_path_from_front<'a, 'src>(
    tokens: &mut &'a TokenStream<'src>,
) -> PResult<Option<S<Path<'src>>>> {
    let mut tok_iter = tokens.iter();
    let mut path = Vec::new();

    let mut last_known_span: Option<Span> = None;

    loop {
        let Some(S(Token::Identifier(name), tok_span)) = tok_iter.next() else {
            let Some(last_known_span) = last_known_span else {
                return Ok(None);
            };

            return Err(ParseError::ExpectedIdentifier(last_known_span.span_after()));
        };

        path.push(S(*name, *tok_span));

        let Some(S(T!("::"), _)) = tok_iter.clone().next() else {
            break;
        };

        last_known_span = tok_iter.next().map(|t| t.1); // consume `::` token
    }

    let remaining_tokens = tok_iter.as_slice();
    let remaining_range = tokens.subslice_range(remaining_tokens).unwrap();

    let path_toks = &tokens[..remaining_range.start];

    *tokens = remaining_tokens;

    Ok(Some(S(path, error_handling::span_of(path_toks).unwrap())))
}
