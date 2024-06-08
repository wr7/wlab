//! Contains rules for the parser. Note: inputs are assumed to not have mismatched/unclosed brackets (these checks should be done in advance).

use std::ops::Deref;

use crate::{error_handling::Spanned as S, lexer::Token, util::SliceExt, T};

use super::{util::NonBracketedIter, Expression, Literal, OpCode, ParseError, Statement};

type PResult<T> = Result<T, ParseError>;

mod bracket_expr;
mod function;

pub use bracket_expr::parse_statement_list;
use wutil::Span;

fn try_parse_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    if tokens.len() == 0 {
        return Ok(None);
    }

    let rules = [
        |tokens| try_parse_literal(tokens),
        |tokens| try_parse_identifier(tokens),
        |tokens| bracket_expr::try_parse_bracket_expr(tokens),
        |tokens| function::try_parse_function_call(tokens),
        |tokens| {
            try_parse_binary_operator(tokens, &[(T!("+"), OpCode::Plus), (T!("-"), OpCode::Minus)])
        },
        |tokens| {
            try_parse_binary_operator(
                tokens,
                &[(T!("*"), OpCode::Asterisk), (T!("/"), OpCode::Slash)],
            )
        },
    ];

    for rule in rules {
        if let Some(item) = rule(tokens)? {
            return Ok(Some(item));
        }
    }

    Err(ParseError::InvalidExpression(
        Span::at(tokens.first().unwrap().1.start).with_end(tokens.last().unwrap().1.end),
    ))
}

/// A statement. This can be either an expression or a few other things.
fn try_parse_statement<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    if tokens.len() == 0 {
        return Ok(None);
    }

    let rules = [
        |tokens| function::try_parse_function(tokens),
        |tokens| try_parse_let(tokens),
        |tokens| try_parse_assign(tokens),
    ];

    for rule in rules {
        if let Some(statement) = rule(tokens)? {
            return Ok(Some(statement));
        }
    }

    if let Some(expr) = try_parse_expr(tokens)? {
        Ok(Some(expr.into()))
    } else {
        return Ok(None);
    }
}

/// A plain identifier
fn try_parse_identifier<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    if let [S(Token::Identifier(ident), _)] = tokens {
        return Ok(Some(Expression::Identifier(ident)));
    }

    Ok(None)
}

/// A literal
fn try_parse_literal<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    match tokens {
        [S(Token::Identifier(ident), _)] => {
            if matches!(ident.chars().next().unwrap(), '0'..='9') {
                return Ok(Some(Expression::Literal(Literal::Number(ident))));
            }
        }
        [S(Token::StringLiteral(lit), _)] => {
            return Ok(Some(Expression::Literal(Literal::String(lit))));
        }
        _ => {}
    }

    Ok(None)
}

/// A variable assignment. Eg `foo = bar * (fizz + buzz)`
fn try_parse_assign<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    let Some(([S(Token::Identifier(var_name), _), S(T!("="), equal_span)], tokens)) =
        tokens.split_first_chunk::<2>()
    else {
        return Ok(None);
    };

    let Some(val) = try_parse_expr(&tokens)? else {
        return Err(ParseError::ExpectedExpression(equal_span.span_after()));
    };

    let span = tokens.first().unwrap().1.start..tokens.last().unwrap().1.end;

    Ok(Some(Statement::Assign(
        &var_name,
        Box::new(S(val, span.into())),
    )))
}

/// A variable initialization. Eg `let foo = bar * (fizz + buzz)`
fn try_parse_let<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    let Some(([S(T!("let"), _), S(Token::Identifier(var_name), name_span)], tokens)) =
        tokens.split_first_chunk::<2>()
    else {
        return Ok(None);
    };

    let Some((S(T!("="), equal_span), tokens)) = tokens.split_first() else {
        return Err(ParseError::ExpectedToken(
            name_span.span_after(),
            &[T!("=")],
        ));
    };

    let Some(val) = try_parse_expr(&tokens)? else {
        return Err(ParseError::ExpectedExpression(equal_span.span_after()));
    };

    let span = tokens.first().unwrap().1.start..tokens.last().unwrap().1.end;

    return Ok(Some(Statement::Let(
        &var_name,
        Box::new(S(val, span.into())),
    )));
}

/// A binary expression. Eg `a + b`
fn try_parse_binary_operator<'a>(
    tokens: &'a [S<Token<'a>>],
    opcodes: &[(Token<'a>, OpCode)],
) -> PResult<Option<Expression<'a>>> {
    for tok in NonBracketedIter::new(tokens).rev() {
        let i = tokens.elem_offset(tok).unwrap();

        for (op_tok, opcode) in opcodes {
            if tok.deref() == op_tok {
                let x = try_parse_expr(&tokens[0..i])?.ok_or_else(|| {
                    ParseError::ExpectedExpression(Span::at(tokens[i].1.start.saturating_sub(1)))
                })?;

                let y = try_parse_expr(&tokens[i + 1..])?
                    .ok_or_else(|| ParseError::ExpectedExpression(tokens[i].1.span_after()))?;

                let x_span = tokens[0].1.start..tokens[i - 1].1.end;
                let y_span = tokens[i + 1].1.start..tokens.last().unwrap().1.end;

                return Ok(Some(Expression::BinaryOperator(
                    Box::new(S(x, x_span.into())),
                    *opcode,
                    Box::new(S(y, y_span.into())),
                )));
            }
        }
    }

    return Ok(None);
}
