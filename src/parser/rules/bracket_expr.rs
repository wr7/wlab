use wutil::iter::IterCloneExt;

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

    for expr in
        NonBracketedIter::new(tokens).split_inclusive(|t| matches!(&t.0, &T!(";") | &T!("}")))
    {
        let Some(expr) = tokens.range_of(expr) else {
            continue;
        };

        let mut expr = &tokens[expr];
        if matches!(expr.last(), Some(&S(T!(";"), _))) {
            expr = &expr[..expr.len() - 1]; // strip trailing semicolon
        }

        if let Some(statement) = try_parse_statement(expr)? {
            items.push(statement)
        }
    }

    Ok(items)
}
