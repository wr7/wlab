//! Contains rules for the parser. Note: inputs are assumed to not have mismatched/unclosed brackets (these checks should be done in advance).

use crate::{
    error_handling::{self, Spanned as S},
    lexer::Token,
    util::SliceExt,
    T,
};

use super::{
    util::NonBracketedIter, Expression, Literal, OpCode, ParseError, Statement, TokenStream,
};

type PResult<T> = Result<T, ParseError>;

mod attributes;
mod bracket_expr;
mod control_flow;
mod function;
mod path;
mod types;

pub use attributes::try_parse_outer_attributes_from_front;
pub use bracket_expr::parse_statement_list;
use wutil::Span;

fn try_parse_expr<'src>(tokens: &TokenStream<'src>) -> PResult<Option<Expression<'src>>> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let rules = [
        |tokens| Ok(try_parse_literal(tokens)),
        |tokens| Ok(try_parse_identifier(tokens)),
        |tokens| control_flow::try_parse_if_expression(tokens),
        |tokens| bracket_expr::try_parse_bracket_expr(tokens),
        |tokens| {
            try_parse_binary_operator(tokens, &[(T!("||"), OpCode::Or), (T!("&&"), OpCode::And)])
        },
        |tokens| {
            try_parse_binary_operator(
                tokens,
                &[
                    (T!(">"), OpCode::Greater),
                    (T!("<"), OpCode::Less),
                    (T!(">="), OpCode::GreaterEqual),
                    (T!("<="), OpCode::LessEqual),
                    (T!("=="), OpCode::Equal),
                    (T!("!="), OpCode::NotEqual),
                ],
            )
        },
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
        error_handling::span_of(tokens).unwrap(),
    ))
}

/// A statement. This can be either an expression or a few other things.
fn try_parse_statement_from_front<'a, 'src>(
    tokens: &'a TokenStream<'src>,
) -> PResult<Option<(Statement<'src>, &'a TokenStream<'src>)>> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let rules = [
        |tokens| function::try_parse_function_from_front(tokens),
        |tokens| Ok(control_flow::try_parse_if_from_front(tokens)?.map(|(ex, r)| (ex.into(), r))),
        |tokens| {
            Ok(
                bracket_expr::try_parse_code_block_from_front(tokens)?.map(|(c, r)| {
                    (
                        Statement::Expression(Expression::CompoundExpression(c.0)),
                        r,
                    )
                }),
            )
        },
        |tokens| Ok(try_parse_let(tokens)?.map(|t| (t, &tokens[Span::at(tokens.len())]))),
        |tokens| Ok(try_parse_assign(tokens)?.map(|t| (t, &tokens[Span::at(tokens.len())]))),
    ];

    for rule in rules {
        if let Some(statement) = rule(tokens)? {
            return Ok(Some(statement));
        }
    }

    if let Some(expr) = try_parse_expr(tokens)? {
        Ok(Some((expr.into(), &tokens[Span::at(tokens.len())])))
    } else {
        Ok(None)
    }
}

/// A plain identifier
fn try_parse_identifier<'src>(tokens: &TokenStream<'src>) -> Option<Expression<'src>> {
    if let [S(Token::Identifier(ident), _)] = tokens {
        return Some(Expression::Identifier(ident));
    }

    None
}

/// A literal
fn try_parse_literal<'src>(tokens: &TokenStream<'src>) -> Option<Expression<'src>> {
    match tokens {
        [S(Token::Identifier(ident), _)] => {
            if ident.chars().next().unwrap().is_ascii_digit() {
                return Some(Expression::Literal(Literal::Number(ident)));
            }
        }
        [S(Token::StringLiteral(lit), _)] => {
            return Some(Expression::Literal(Literal::String(lit.clone())));
        }
        _ => {}
    }

    None
}

/// A variable assignment. Eg `foo = bar * (fizz + buzz)`
fn try_parse_assign<'src>(tokens: &TokenStream<'src>) -> PResult<Option<Statement<'src>>> {
    let Some(([S(Token::Identifier(var_name), _), S(T!("="), equal_span)], tokens)) =
        tokens.split_first_chunk::<2>()
    else {
        return Ok(None);
    };

    let Some(val) = try_parse_expr(tokens)? else {
        return Err(ParseError::ExpectedExpression(equal_span.span_after()));
    };

    let span = error_handling::span_of(tokens).unwrap();

    Ok(Some(Statement::Assign(var_name, Box::new(S(val, span)))))
}

/// A variable initialization. Eg `let foo = bar * (fizz + buzz)`
fn try_parse_let<'src>(tokens: &TokenStream<'src>) -> PResult<Option<Statement<'src>>> {
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

    let Some(val) = try_parse_expr(tokens)? else {
        return Err(ParseError::ExpectedExpression(equal_span.span_after()));
    };

    let span = error_handling::span_of(tokens).unwrap();

    return Ok(Some(Statement::Let(var_name, Box::new(S(val, span)))));
}

/// A binary expression. Eg `a + b`
fn try_parse_binary_operator<'src>(
    tokens: &TokenStream<'src>,
    opcodes: &'static [(Token<'static>, OpCode)],
) -> PResult<Option<Expression<'src>>> {
    for tok in NonBracketedIter::new(tokens).rev() {
        let i = tokens.elem_offset(tok).unwrap();

        for (op_tok, opcode) in opcodes {
            if &**tok == op_tok {
                let x = try_parse_expr(&tokens[..i])?.ok_or_else(|| {
                    ParseError::ExpectedExpression(Span::at(tokens[i].1.start.saturating_sub(1)))
                })?;

                let y = try_parse_expr(&tokens[i + 1..])?
                    .ok_or_else(|| ParseError::ExpectedExpression(tokens[i].1.span_after()))?;

                let x_span = error_handling::span_of(&tokens[..i]).unwrap();
                let y_span = error_handling::span_of(&tokens[i + 1..]).unwrap();

                return Ok(Some(Expression::BinaryOperator(
                    Box::new(S(x, x_span)),
                    *opcode,
                    Box::new(S(y, y_span)),
                )));
            }
        }
    }

    Ok(None)
}
