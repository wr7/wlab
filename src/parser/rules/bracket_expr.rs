use crate::{
    error_handling::Spanned as S,
    lexer::{BracketType, Token},
    parser::{
        error::check_brackets, rules::try_parse_expr, util::TokenSplit, Expression, Statement,
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

    for (mut stmnt, separator) in TokenSplit::new(tokens, |t| matches!(&t, &T!(";") | &T!("}"))) {
        if matches!(separator, Some(S(T!("}"), _))) {
            let stmnt_idx = tokens.subslice_range(stmnt).unwrap();
            stmnt = &tokens[stmnt_idx.start..stmnt_idx.end + 1]; // Include closing bracket
        }

        if let Some(statement) = try_parse_statement(stmnt)? {
            items.push(statement)
        }
    }

    Ok(items)
}
