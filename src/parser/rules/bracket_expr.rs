use crate::{
    error_handling::Spanned as S,
    lexer::{BracketType, Token},
    parser::{
        error::check_brackets, rules::try_parse_expr, util::NonBracketedIter, Expression, Statement,
    },
    util::SliceExt,
    T,
};

use super::{try_parse_statement, PResult};

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
pub fn try_parse_bracket_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    let Some((S(Token::OpenBracket(opening_type), _), tokens)) = tokens.split_first() else {
        return Ok(None);
    };

    let Some((S(Token::CloseBracket(closing_type), _), tokens)) = tokens.split_last() else {
        return Ok(None);
    };

    if opening_type != closing_type || check_brackets(tokens).is_err() {
        return Ok(None);
    }

    if *opening_type == BracketType::Curly {
        let statements = parse_statement_list(tokens)?;
        Ok(Some(Expression::CompoundExpression(statements)))
    } else {
        Ok(try_parse_expr(tokens)?)
    }
}

pub fn parse_statement_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<Statement<'a>>> {
    let mut items = Vec::new();

    let mut statement_start = 0;

    for t @ S(tok, _) in NonBracketedIter::new(tokens) {
        let i = tokens.elem_offset(t).unwrap();

        if !matches!(tok, &T!(";") | &T!("}")) {
            continue;
        }

        let statement = match tok {
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
