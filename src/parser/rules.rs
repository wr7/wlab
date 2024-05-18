use std::ops::Deref;

use crate::{
    error_handling::Spanned,
    lexer::{BracketType, Token},
    util::IterExt,
    T,
};

use super::{Expression, OpCode, ParseError, Statement};

type PResult<T> = Result<T, Spanned<ParseError>>;

pub fn parse_statement_list<'a>(tokens: &'a [Spanned<Token<'a>>]) -> PResult<Vec<Statement<'a>>> {
    let mut items = Vec::new();

    let mut bracket_level = 0u32;
    let mut statement_start = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if matches!(tok.deref(), Token::OpenBracket(_)) {
            bracket_level += 1;
        } else if matches!(tok.deref(), Token::CloseBracket(_)) {
            bracket_level = bracket_level
                .checked_sub(1)
                .ok_or(Spanned(ParseError::UnmatchedBracket, tok.1.clone()))?;
        }

        if bracket_level != 0 {
            continue;
        }

        if !matches!(tok.deref(), &T!(";") | &T!("}")) {
            continue;
        }

        if tok.deref() == &T!(";") && statement_start != i {
            items.push(try_parse_statement(&tokens[statement_start..i]).unwrap());
        } else if tok.deref() == &T!("}") {
            items.push(try_parse_statement(&tokens[statement_start..=i]).unwrap());
        }

        statement_start = i + 1;
    }

    if statement_start != tokens.len() {
        items.push(try_parse_statement(&tokens[statement_start..]).unwrap());
    }

    Ok(items)
}

/// A plain identifier
fn try_parse_ident<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Expression<'a>> {
    if let Token::Identifier(ident) = tokens.first()?.deref() {
        if tokens.len() == 1 {
            return Some(Expression::Identifier(ident));
        }
    }

    None
}

/// A statement surrounded in brackets eg `(foo + bar)` or `{biz+bang; do_thing*f}`. The latter case is a compound statement
fn try_parse_bracket_expr<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Expression<'a>> {
    let Token::OpenBracket(bracket_type) = tokens.first()?.deref() else {
        return None;
    };

    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate() {
        let tok = tok.deref();

        if matches!(tok, Token::OpenBracket(_)) {
            bracket_level += 1;
        } else if matches!(tok, Token::CloseBracket(_)) {
            bracket_level -= 1;
            if bracket_level == 0 {
                assert_eq!(tok, &Token::CloseBracket(*bracket_type));
                if *bracket_type == BracketType::Curly {
                    return Some(Expression::CompoundExpression(
                        parse_statement_list(&tokens[1..i]).unwrap(),
                    ));
                } else {
                    return try_parse_expr(&tokens[1..i]);
                }
            }
        }
    }

    dbg!(tokens);
    panic!("Expected closing bracket, reached end of expression")
}

/// A function. Eg `fn foo() {let x = ten; x}`
fn try_parse_function<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Statement<'a>> {
    let mut token_iter = tokens.iter().enumerate();

    let Some([&T!("fn"), Token::Identifier(fn_name)]) =
        (&mut token_iter).map(|t| t.1.deref()).collect_n::<2>()
    else {
        return None;
    };

    let Some([T!("("), T!(")")]) = (&mut token_iter).map(|t| t.1.deref()).collect_n::<2>() else {
        panic!("Expected parenthesis");
    };

    let (body_start, _) = token_iter.next().unwrap();

    let Expression::CompoundExpression(body) =
        try_parse_bracket_expr(&tokens[body_start..]).unwrap()
    else {
        unreachable!()
    };

    Some(Statement::Function(&fn_name, body))
}

/// A variable assignment. Eg `foo = bar * (fizz + buzz)`
fn try_parse_assign<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Statement<'a>> {
    let Some([Token::Identifier(var_name), T!("=")]) =
        tokens.iter().map(|t| t.deref()).collect_n::<2>()
    else {
        return None;
    };

    if tokens.len() <= 2 {
        panic!()
    }

    let val = try_parse_expr(&tokens[2..]);

    Some(Statement::Assign(&var_name, Box::new(val.unwrap())))
}

/// A variable initialization. Eg `let foo = bar * (fizz + buzz)`
fn try_parse_let<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Statement<'a>> {
    let mut token_iter = tokens.iter().enumerate();

    let Some([&T!("let"), Token::Identifier(var_name)]) =
        (&mut token_iter).map(|t| t.1.deref()).collect_n::<2>()
    else {
        return None;
    };

    let (equal_idx, Spanned(T!("="), _)) = token_iter.next().unwrap() else {
        panic!()
    };

    let val = try_parse_expr(&tokens[equal_idx + 1..]).unwrap();

    return Some(Statement::Let(&var_name, Box::new(val)));
}

/// A binary expression. Eg `a + b`
fn try_parse_bin<'a>(
    tokens: &'a [Spanned<Token<'a>>],
    opcodes: &[(Token<'a>, OpCode)],
) -> Option<Expression<'a>> {
    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate().rev() {
        let tok = tok.deref();

        if matches!(tok, Token::OpenBracket(_)) {
            assert_ne!(bracket_level, 0);
            bracket_level -= 1;
        } else if matches!(tok, Token::CloseBracket(_)) {
            bracket_level += 1;
        }

        if bracket_level != 0 {
            continue;
        }

        for (ttok, opcode) in opcodes {
            if tok == ttok {
                let x = try_parse_expr(&tokens[0..i]);
                let y = try_parse_expr(&tokens[(i + 1)..]);
                return Some(Expression::BinaryOperator(
                    Box::new(x.unwrap()),
                    *opcode,
                    Box::new(y.unwrap()),
                ));
            }
        }
    }

    return None;
}

/// A statement. This can be either an expression or a few other things.
fn try_parse_statement<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Statement<'a>> {
    if tokens.len() == 0 {
        return None;
    }

    let rules = [
        |tokens| try_parse_function(tokens),
        |tokens| try_parse_let(tokens),
        |tokens| try_parse_assign(tokens),
    ];

    for rule in rules {
        if let Some(statement) = rule(tokens) {
            return Some(statement);
        }
    }

    try_parse_expr(tokens).map(|e| e.into())
}

fn try_parse_expr<'a>(tokens: &'a [Spanned<Token<'a>>]) -> Option<Expression<'a>> {
    if tokens.len() == 0 {
        return None;
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
        if let Some(item) = rule(tokens) {
            return Some(item);
        }
    }

    panic!("Invalid syntax")
}
