use wutil::iter::IterExt;

use crate::{
    error_handling::Spanned as S,
    lexer::{BracketType, Token},
    parser::{
        rules::try_parse_expr,
        util::{NonBracketedIter, TokenSplit},
        CodeBlock, Expression, Statement,
    },
    util::SliceExt,
    T,
};

use super::{try_parse_statement, PResult};

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
pub fn try_parse_bracket_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    let mut nb_iter = NonBracketedIter::new(tokens);

    let Some([S(Token::OpenBracket(opening_type), _), close_bracket]) = nb_iter.collect_n() else {
        return Ok(None);
    };

    let closing_idx = tokens.elem_offset(close_bracket).unwrap();

    if closing_idx != tokens.len() - 1 {
        return Ok(None);
    }

    if *opening_type == BracketType::Curly {
        let body = parse_statement_list(&tokens[1..closing_idx])?;

        let trailing_semicolon = if let S(T!(";"), s) = &tokens[closing_idx - 1] {
            Some(*s)
        } else {
            None
        };

        Ok(Some(Expression::CompoundExpression(CodeBlock {
            body,
            trailing_semicolon,
        })))
    } else {
        Ok(try_parse_expr(&tokens[1..closing_idx])?)
    }
}

pub fn parse_statement_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<S<Statement<'a>>>> {
    let mut items = Vec::new();

    for (mut stmnt, separator) in TokenSplit::new(tokens, |t| matches!(&t, &T!(";") | &T!("}"))) {
        if matches!(separator, Some(S(T!("}"), _))) {
            let stmnt_idx = tokens.subslice_range(stmnt).unwrap();
            stmnt = &tokens[stmnt_idx.start..=stmnt_idx.end]; // Include closing bracket
        }

        if let Some(statement) = try_parse_statement(stmnt)? {
            let span = stmnt.first().unwrap().1.start..stmnt.last().unwrap().1.end;
            items.push(S(statement, span.into()));
        }
    }

    Ok(items)
}
