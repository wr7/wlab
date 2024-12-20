//! Contains rules for the parser. Note: inputs are assumed to not have mismatched/unclosed brackets (these checks should be done in advance).

use wutil::Span;

use crate::{
    error_handling::{self, Diagnostic, Spanned as S},
    lexer::Token,
    parser::{
        ast::{Expression, Literal, OpCode, Statement},
        error,
        util::NonBracketedIter,
        TokenStream,
    },
    T,
};

type PResult<T> = Result<T, Diagnostic>;

mod attributes;
mod bracket_expr;
mod control_flow;
mod function;
mod path;
mod struct_;
mod types;

pub use attributes::try_parse_outer_attributes_from_front;
pub use bracket_expr::parse_statement_list;

use super::macros::match_tokens;

fn try_parse_expr<'src>(tokens: &TokenStream<'src>) -> PResult<Option<Expression<'src>>> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let rules = [
        |tokens| control_flow::try_parse_if_expression(tokens),
        |tokens| control_flow::try_parse_break_or_return(tokens),
        |tokens| Ok(try_parse_literal(tokens)),
        |tokens| Ok(try_parse_identifier(tokens)),
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
        |tokens| {
            try_parse_binary_operator(tokens, &[(T!("+"), OpCode::Plus), (T!("-"), OpCode::Minus)])
        },
        |tokens| {
            try_parse_binary_operator(
                tokens,
                &[(T!("*"), OpCode::Asterisk), (T!("/"), OpCode::Slash)],
            )
        },
        |tokens| struct_::try_parse_field_access(tokens),
        |tokens| control_flow::try_parse_loop(tokens),
        |tokens| function::try_parse_function_call(tokens),
        |tokens| struct_::try_parse_struct_initializer(tokens),
    ];

    for rule in rules {
        if let Some(item) = rule(tokens)? {
            return Ok(Some(item));
        }
    }

    Err(error::invalid_expression(
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
        |tokens| {
            Ok(control_flow::try_parse_loop_from_front(tokens)?
                .map(|(ex, r)| (Statement::from(ex), r)))
        },
        |tokens| struct_::try_parse_struct_from_front(tokens),
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
    if matches!(tokens.first(), Some(S(T!("let"), _))) {
        return Ok(None);
    }

    let Some((equal_sign, equals_idx)) = NonBracketedIter::new(tokens)
        .find(|t| ***t == T!("="))
        .and_then(|t| Some(t).zip(tokens.elem_offset(t)))
    else {
        return Ok(None);
    };

    let lhs_tokens = &tokens[..equals_idx];
    let lhs = try_parse_expr(lhs_tokens)?
        .ok_or_else(|| error::expected_expression(equal_sign.1.span_at()))?;
    let lhs = S(lhs, error_handling::span_of(lhs_tokens).unwrap());

    let rhs_tokens = &tokens[equals_idx + 1..];
    let rhs = try_parse_expr(rhs_tokens)?
        .ok_or_else(|| error::expected_expression(equal_sign.1.span_after()))?;
    let rhs = S(rhs, error_handling::span_of(rhs_tokens).unwrap());

    Ok(Some(Statement::Assign { lhs, rhs }))
}

/// A variable initialization. Eg `let foo = bar * (fizz + buzz)`
fn try_parse_let<'src>(tokens: &TokenStream<'src>) -> PResult<Option<Statement<'src>>> {
    match_tokens! {
        tokens: {
            required(token("let"));

            token("mut") @ mut_tok;

            required {
                ident() @ (name, S(_, name_span));

                token("=") else {
                    return Err(error::expected_token(name_span.span_after(), &[T!("=")]));
                } @ (&S(_, equal_span));
            };

            required(do_(|tokens| {
                try_parse_expr(tokens)?.zip(error_handling::span_of(tokens))
            })) else {
                return Err(error::expected_expression(equal_span.span_after()));
            } @ (val, val_span);
        } => {

            Ok(Some(Statement::Let {
                name: S(name, *name_span),
                value: Box::new(S(val, val_span)),
                mutable: mut_tok.is_some(),
            }))
        }
    }
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
                    error::expected_expression(Span::at(tokens[i].1.start.saturating_sub(1)))
                })?;

                let y = try_parse_expr(&tokens[i + 1..])?
                    .ok_or_else(|| error::expected_expression(tokens[i].1.span_after()))?;

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
