use crate::{
    error_handling::{self, Spanned as S},
    parser::{
        ast::{CodeBlock, Expression, Statement},
        error,
        rules::{bracket_expr::try_parse_code_block_from_front, try_parse_expr, PResult},
        util::NonBracketedIter,
        TokenStream,
    },
    util::SliceExt,
    T,
};

pub fn try_parse_if_expression<'src>(
    tokens: &TokenStream<'src>,
) -> PResult<Option<Expression<'src>>> {
    let Some((expr, trailing_tokens)) = try_parse_if_from_front(tokens)? else {
        return Ok(None);
    };

    if let Some(trailing_tokens_span) = error_handling::span_of(trailing_tokens) {
        return Err(error::unexpected_tokens(trailing_tokens_span));
    }

    Ok(Some(expr))
}

pub fn try_parse_if_from_front<'a, 'src>(
    tokens: &'a TokenStream<'src>,
) -> PResult<Option<(Expression<'src>, &'a TokenStream<'src>)>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some(S(T!("if"), if_span)) = nb_iter.next() else {
        return Ok(None);
    };

    let Some(left_bracket) = nb_iter.find(|t| ***t == T!("{")) else {
        return Err(error::missing_block(*if_span));
    };

    let left_idx = tokens.elem_offset(left_bracket).unwrap();

    let Some(condition) = try_parse_expr(&tokens[1..left_idx])? else {
        return Err(error::expected_expression(if_span.span_after()));
    };
    let condition_span = error_handling::span_of(&tokens[1..left_idx]).unwrap();

    let Some((block, remaining_tokens)) = try_parse_code_block_from_front(&tokens[left_idx..])?
    else {
        unreachable!()
    };

    let Some(S(T!("else"), else_span)) = remaining_tokens.first() else {
        return Ok(Some((
            Expression::If {
                condition: Box::new(S(condition, condition_span)),
                block,
                else_block: None,
            },
            remaining_tokens,
        )));
    };

    if let Some(S(T!("if"), _)) = remaining_tokens.get(1) {
        let else_if_span = error_handling::span_of(&remaining_tokens[1..]).unwrap();

        let Some((else_if, remaining_tokens)) = try_parse_if_from_front(&remaining_tokens[1..])?
        else {
            unreachable!()
        };

        let else_block = CodeBlock {
            body: vec![S(Statement::Expression(else_if), else_if_span)],
            trailing_semicolon: None,
        };

        let else_block = S(else_block, else_if_span);

        return Ok(Some((
            Expression::If {
                condition: Box::new(S(condition, condition_span)),
                block,
                else_block: Some(else_block),
            },
            remaining_tokens,
        )));
    }

    let else_block_span =
        error_handling::span_of(&remaining_tokens[1..]).unwrap_or(else_span.span_after());

    let Some((else_block, remaining_tokens)) =
        try_parse_code_block_from_front(&remaining_tokens[1..])?
    else {
        return Err(error::expected_body(else_block_span));
    };

    Ok(Some((
        Expression::If {
            condition: Box::new(S(condition, condition_span)),
            block,
            else_block: Some(else_block),
        },
        remaining_tokens,
    )))
}
