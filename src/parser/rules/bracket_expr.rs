use wutil::iter::IterExt;

use crate::{
    error_handling::{self, Spanned as S},
    parser::{
        ast::{CodeBlock, Expression, Statement},
        rules::{self, try_parse_statement_from_front, PResult},
        util::{NonBracketedIter, TokenSplit},
        TokenStream,
    },
    T,
};

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
pub fn try_parse_bracket_expr<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<Expression<'src>>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    if let Some([S(T!("("), _), close_bracket]) = nb_iter.collect_n() {
        let close_idx = tokens.elem_offset(close_bracket).unwrap();

        // Check for trailing tokens
        if close_idx != tokens.len() - 1 {
            return Ok(None); // Will resolve to an error elsewhere
        }

        return rules::try_parse_expr(&tokens[1..close_idx]);
    }

    let Some((code_block, &[])) = try_parse_code_block_from_front(tokens)? else {
        return Ok(None);
    };

    Ok(Some(Expression::CompoundExpression(code_block.0)))
}

/// A code block eg `{biz+bang; do_thing()}`
pub fn try_parse_code_block_from_front<'a, 'src>(
    tokens: &'a TokenStream<'src>,
) -> PResult<Option<(S<CodeBlock<'src>>, &'a TokenStream<'src>)>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some([S(T!("{"), _), close_bracket]) = nb_iter.collect_n() else {
        return Ok(None);
    };

    let closing_idx = tokens.elem_offset(close_bracket).unwrap();

    let body = parse_statement_list(&tokens[1..closing_idx])?;
    let span = error_handling::span_of(&tokens[0..=closing_idx]).unwrap();

    let trailing_semicolon = if let S(T!(";"), s) = &tokens[closing_idx - 1] {
        Some(*s)
    } else {
        None
    };

    Ok(Some((
        S(
            CodeBlock {
                body,
                trailing_semicolon,
            },
            span,
        ),
        &tokens[closing_idx + 1..],
    )))
}

pub fn parse_statement_list<'src>(tokens: &TokenStream<'src>) -> PResult<Vec<S<Statement<'src>>>> {
    let mut items = Vec::new();

    let mut queued_tokens = None;
    let mut token_split = TokenSplit::new(tokens, |t| matches!(&t, &T!(";"))).map(|v| v.0);

    while let Some(stmnt) = queued_tokens.take().or_else(|| token_split.next()) {
        if let Some((statement, remaining_tokens)) = try_parse_statement_from_front(stmnt)? {
            let remaining_token_idx = stmnt.subslice_range(remaining_tokens).unwrap();

            let span = error_handling::span_of(&stmnt[..remaining_token_idx.start]).unwrap();
            items.push(S(statement, span));

            if !remaining_tokens.is_empty() {
                queued_tokens = Some(remaining_tokens);
            }
        }
    }

    Ok(items)
}
