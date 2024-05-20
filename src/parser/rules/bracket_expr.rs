use std::ops::Deref as _;

use crate::{
    error_handling::Spanned as S,
    lexer::{BracketType, Token},
    parser::{rules::try_parse_expr, Expression, Statement},
    T,
};

use super::{try_parse_statement, PResult};

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
pub fn try_parse_bracket_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    let Some(S(Token::OpenBracket(bracket_type), _)) = tokens.first() else {
        return Ok(None);
    };

    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if matches!(tok.deref(), Token::OpenBracket(_)) {
            bracket_level += 1;
            continue;
        }
        if !matches!(tok.deref(), Token::CloseBracket(_)) {
            continue;
        }

        bracket_level -= 1;
        if bracket_level != 0 {
            continue;
        }

        if tokens.len() > i + 1 {
            return Ok(None);
        }

        if *bracket_type == BracketType::Curly {
            return Ok(Some(Expression::CompoundExpression(parse_statement_list(
                &tokens[1..i],
            )?)));
        } else {
            return try_parse_expr(&tokens[1..i]);
        }
    }

    unreachable!();
}

pub fn parse_statement_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<Statement<'a>>> {
    let mut items = Vec::new();

    let mut bracket_level = 0u32;
    let mut statement_start = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if matches!(tok.deref(), Token::OpenBracket(_)) {
            bracket_level += 1;
        } else if matches!(tok.deref(), Token::CloseBracket(_)) {
            bracket_level -= 1;
        }

        if bracket_level != 0 {
            continue;
        }

        if !matches!(tok.deref(), &T!(";") | &T!("}")) {
            continue;
        }

        let statement = match tok.deref() {
            T!(";") => &tokens[statement_start..i],
            T!("}") => &tokens[statement_start..=i],
            _ => continue,
        };

        try_parse_statement(statement)?.map(|s| items.push(s));

        statement_start = i + 1;
    }

    // Parse trailing statement without semicolon
    if statement_start < tokens.len() {
        items.push(try_parse_statement(&tokens[statement_start..])?.unwrap());
    }

    Ok(items)
}
