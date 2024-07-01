use wutil::iter::IterExt;

use crate::{
    error_handling::Spanned as S,
    lexer::Token,
    parser::{
        rules::{self},
        util::{NonBracketedIter, TokenSplit},
        CodeBlock, Expression, Statement,
    },
    util::SliceExt,
    T,
};

use super::{try_parse_statement_from_front, PResult};

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
pub fn try_parse_bracket_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
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

    Ok(Some(Expression::CompoundExpression(code_block)))
}

/// A code block eg `{biz+bang; do_thing()}`
pub fn try_parse_code_block_from_front<'a>(
    tokens: &'a [S<Token<'a>>],
) -> PResult<Option<(CodeBlock<'a>, &'a [S<Token<'a>>])>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some([S(T!("{"), _), close_bracket]) = nb_iter.collect_n() else {
        return Ok(None);
    };

    let closing_idx = tokens.elem_offset(close_bracket).unwrap();

    let body = parse_statement_list(&tokens[1..closing_idx])?;

    let trailing_semicolon = if let S(T!(";"), s) = &tokens[closing_idx - 1] {
        Some(*s)
    } else {
        None
    };

    Ok(Some((
        CodeBlock {
            body,
            trailing_semicolon,
        },
        &tokens[closing_idx + 1..],
    )))
}

pub fn parse_statement_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<S<Statement<'a>>>> {
    let mut items = Vec::new();

    let mut queued_tokens = None;
    let mut token_split = TokenSplit::new(tokens, |t| matches!(&t, &T!(";"))).map(|v| v.0);

    while let Some(stmnt) = queued_tokens.take().or_else(|| token_split.next()) {
        if let Some((statement, remaining_tokens)) = try_parse_statement_from_front(stmnt)? {
            let span = stmnt.first().unwrap().1.start..stmnt.last().unwrap().1.end;
            items.push(S(statement, span.into()));

            if !remaining_tokens.is_empty() {
                queued_tokens = Some(remaining_tokens)
            }
        }
    }

    Ok(items)
}
