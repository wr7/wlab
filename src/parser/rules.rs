use std::ops::Deref;

use crate::{
    error_handling::Spanned as S,
    lexer::{BracketType, Token},
    T,
};

use super::{Expression, OpCode, ParseError, Statement};

type PResult<T> = Result<T, ParseError>;

pub fn parse_statement_list<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Vec<Statement<'a>>> {
    let mut items = Vec::new();

    let mut bracket_level = 0u32;
    let mut statement_start = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if matches!(tok.deref(), Token::OpenBracket(_)) {
            bracket_level += 1;
        } else if matches!(tok.deref(), Token::CloseBracket(_)) {
            bracket_level = bracket_level
                .checked_sub(1)
                .ok_or(ParseError::UnmatchedBracket(tok.1.clone()))?;
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

/// A plain identifier
fn try_parse_ident<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    if let [S(Token::Identifier(ident), _)] = tokens {
        return Ok(Some(Expression::Identifier(ident)));
    }

    Ok(None)
}

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
fn try_parse_bracket_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    let Some(S(Token::OpenBracket(bracket_type), open_bracket_pos)) = tokens.first() else {
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

        if tok.deref() != &Token::CloseBracket(*bracket_type) {
            return Err(ParseError::MismatchedBrackets(
                open_bracket_pos.clone(),
                tok.1.clone(),
            ));
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

    Err(
        // TODO: enforce certain mismatched brackets before this
        ParseError::UnmatchedBracket(open_bracket_pos.clone()),
    )
}

/// A function. Eg `fn foo() {let x = ten; x}`
fn try_parse_function<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    let Some(([S(T!("fn"), _), S(Token::Identifier(fn_name), name_span)], tokens)) =
        tokens.split_first_chunk::<2>()
    else {
        return Ok(None);
    };

    let Some(([S(T!("("), _), S(T!(")"), rparen)], tokens)) = tokens.split_first_chunk::<2>()
    else {
        return Err(ParseError::ExpectedParameters(name_span.end..name_span.end));
    };

    let Some(Expression::CompoundExpression(body)) = try_parse_bracket_expr(tokens)? else {
        return Err(ParseError::ExpectedBody(rparen.end..rparen.end));
    };

    Ok(Some(Statement::Function(&fn_name, body)))
}

/// A variable assignment. Eg `foo = bar * (fizz + buzz)`
fn try_parse_assign<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    let Some(([S(Token::Identifier(var_name), _), S(T!("="), equal_span)], tokens)) =
        tokens.split_first_chunk::<2>()
    else {
        return Ok(None);
    };

    let Some(val) = try_parse_expr(&tokens)? else {
        return Err(ParseError::ExpectedExpression(
            equal_span.end..equal_span.end,
        ));
    };

    Ok(Some(Statement::Assign(&var_name, Box::new(val))))
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
            name_span.end..name_span.end,
            T!("="),
        ));
    };

    let Some(val) = try_parse_expr(&tokens)? else {
        return Err(ParseError::ExpectedExpression(
            equal_span.end..equal_span.end,
        ));
    };

    return Ok(Some(Statement::Let(&var_name, Box::new(val))));
}

/// A binary expression. Eg `a + b`
fn try_parse_bin<'a>(
    tokens: &'a [S<Token<'a>>],
    opcodes: &[(Token<'a>, OpCode)],
) -> PResult<Option<Expression<'a>>> {
    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate().rev() {
        if matches!(tok.deref(), Token::OpenBracket(_)) {
            if bracket_level == 0 {
                return Err(ParseError::UnmatchedBracket(tok.1.clone()));
            }
            bracket_level -= 1;
        } else if matches!(tok.deref(), Token::CloseBracket(_)) {
            bracket_level += 1;
        }

        if bracket_level != 0 {
            continue;
        }

        for (ttok, opcode) in opcodes {
            if tok.deref() == ttok {
                let x = try_parse_expr(&tokens[0..i])?.ok_or(ParseError::ExpectedExpression(
                    tokens[i].1.start - 1..tokens[i].1.start - 1,
                ))?;

                let y = try_parse_expr(&tokens[i + 1..])?.ok_or(ParseError::ExpectedExpression(
                    tokens[i].1.end..tokens[i].1.end,
                ))?;

                return Ok(Some(Expression::BinaryOperator(
                    Box::new(x),
                    *opcode,
                    Box::new(y),
                )));
            }
        }
    }

    return Ok(None);
}

/// A statement. This can be either an expression or a few other things.
fn try_parse_statement<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Statement<'a>>> {
    if tokens.len() == 0 {
        return Ok(None);
    }

    let rules = [
        |tokens| try_parse_function(tokens),
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

fn try_parse_expr<'a>(tokens: &'a [S<Token<'a>>]) -> PResult<Option<Expression<'a>>> {
    if tokens.len() == 0 {
        return Ok(None);
    }

    let rules = [
        |tokens| try_parse_ident(tokens),
        |tokens| try_parse_bracket_expr(tokens),
        |tokens| try_parse_bin(tokens, &[(T!("+"), OpCode::Plus), (T!("-"), OpCode::Minus)]),
        |tokens| {
            try_parse_bin(
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
        tokens.first().unwrap().1.start..tokens.last().unwrap().1.end,
    ))
}
