use std::process::abort;

use crate::{
    lexer::{BracketType, Token},
    T,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Let(&'a str, Box<Expression<'a>>),
    Assign(&'a str, Box<Expression<'a>>),
    Function(&'a str, Vec<Statement<'a>>),
}

impl<'a> From<Expression<'a>> for Statement<'a> {
    fn from(expr: Expression<'a>) -> Self {
        Statement::Expression(expr)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression<'a> {
    Identifier(&'a str),
    BinaryOperator(Box<Self>, OpCode, Box<Self>),
    CompoundExpression(Vec<Statement<'a>>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Plus,
    Minus,
    Asterisk,
    Slash,
}

pub fn try_parse_ident<'a>(tokens: &'a [Token<'a>]) -> Option<Expression<'a>> {
    if let Token::Identifier(ident) = tokens.first()? {
        if tokens.len() == 1 {
            return Some(Expression::Identifier(ident));
        }
    }

    None
}

pub fn try_parse_bracket_expr<'a>(tokens: &'a [Token<'a>]) -> Option<Expression<'a>> {
    let Token::OpenBracket(bracket_type) = tokens.first()? else {
        return None;
    };

    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if matches!(tok, Token::OpenBracket(_)) {
            bracket_level += 1;
        } else if matches!(tok, Token::CloseBracket(_)) {
            bracket_level -= 1;
            if bracket_level == 0 {
                assert_eq!(tok, &Token::CloseBracket(*bracket_type));
                if *bracket_type == BracketType::Curly {
                    return Some(Expression::CompoundExpression(parse_statement_list(
                        &tokens[1..i],
                    )));
                } else {
                    return try_parse_expr(&tokens[1..i]);
                }
            }
        }
    }

    dbg!(tokens);
    panic!("Expected closing bracket, reached end of expression")
}

pub fn try_parse_function<'a>(tokens: &'a [Token<'a>]) -> Option<Statement<'a>> {
    let mut token_iter = tokens.iter().enumerate();

    let [&T!("fn"), Token::Identifier(fn_name)] = [token_iter.next()?.1, token_iter.next()?.1]
    else {
        return None;
    };
    let [T!("("), T!(")")] = [token_iter.next()?.1, token_iter.next()?.1] else {
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

pub fn try_parse_assign<'a>(tokens: &'a [Token<'a>]) -> Option<Statement<'a>> {
    let mut token_iter = tokens.iter();

    let [Token::Identifier(var_name), T!("=")] = [token_iter.next()?, token_iter.next()?] else {
        return None;
    };

    if tokens.len() <= 2 {
        panic!()
    }

    let val = try_parse_expr(&tokens[2..]);

    Some(Statement::Assign(&var_name, Box::new(val.unwrap())))
}

pub fn try_parse_let<'a>(tokens: &'a [Token<'a>]) -> Option<Statement<'a>> {
    let mut token_iter = tokens.iter().enumerate();

    let [&T!("let"), Token::Identifier(var_name)] = [token_iter.next()?.1, token_iter.next()?.1]
    else {
        return None;
    };

    let (equal_idx, T!("=")) = token_iter.next().unwrap() else {
        panic!()
    };

    let val = try_parse_expr(&tokens[equal_idx + 1..]).unwrap();

    return Some(Statement::Let(&var_name, Box::new(val)));
}

pub fn try_parse_bin<'a>(
    tokens: &'a [Token<'a>],
    opcodes: &[(Token<'a>, OpCode)],
) -> Option<Expression<'a>> {
    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate().rev() {
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
                if x.is_none() || y.is_none() {
                    abort();
                }
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

pub fn parse_statement_list<'a>(tokens: &'a [Token<'a>]) -> Vec<Statement<'a>> {
    let mut items = Vec::new();

    let mut bracket_level = 0;
    let mut statement_start = 0;

    for (i, tok) in tokens.iter().enumerate() {
        if matches!(tok, Token::OpenBracket(_)) {
            bracket_level += 1;
        } else if matches!(tok, Token::CloseBracket(_)) {
            assert_ne!(bracket_level, 0);
            bracket_level -= 1;
        }

        if bracket_level != 0 {
            continue;
        }

        if !matches!(tok, &T!(";") | &T!("}")) {
            continue;
        }

        if tok == &T!(";") && statement_start != i {
            items.push(try_parse_statement(&tokens[statement_start..i]).unwrap());
        } else if tok == &T!("}") {
            items.push(try_parse_statement(&tokens[statement_start..=i]).unwrap());
        }

        statement_start = i + 1;
    }

    if statement_start != tokens.len() {
        items.push(try_parse_statement(&tokens[statement_start..]).unwrap());
    }

    items
}

pub fn try_parse_statement<'a>(tokens: &'a [Token<'a>]) -> Option<Statement<'a>> {
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

pub fn try_parse_expr<'a>(tokens: &'a [Token<'a>]) -> Option<Expression<'a>> {
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

    dbg!(tokens);
    abort();
    panic!("Invalid syntax")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::{Lexer, LexerError};

    #[test]
    fn test_parser() {
        let tokens: Result<Vec<Token>, LexerError> =
            Lexer::new("foo + bar/biz*(bang-zig)+(his*fan)/four+se/(ki*kp)").collect();
        let tokens: Vec<Token> = tokens.unwrap();

        let ast = try_parse_expr(&tokens).unwrap();
        let expected_ast = Expression::BinaryOperator(
            Box::new(Expression::BinaryOperator(
                Box::new(Expression::BinaryOperator(
                    Box::new(Expression::Identifier("foo")),
                    OpCode::Plus,
                    Box::new(Expression::BinaryOperator(
                        Box::new(Expression::BinaryOperator(
                            Box::new(Expression::Identifier("bar")),
                            OpCode::Slash,
                            Box::new(Expression::Identifier("biz")),
                        )),
                        OpCode::Asterisk,
                        Box::new(Expression::BinaryOperator(
                            Box::new(Expression::Identifier("bang")),
                            OpCode::Minus,
                            Box::new(Expression::Identifier("zig")),
                        )),
                    )),
                )),
                OpCode::Plus,
                Box::new(Expression::BinaryOperator(
                    Box::new(Expression::Identifier("his")),
                    OpCode::Asterisk,
                    Box::new(Expression::Identifier("fan")),
                )),
            )),
            OpCode::Plus,
            Box::new(Expression::BinaryOperator(
                Box::new(Expression::Identifier("se")),
                OpCode::Slash,
                Box::new(Expression::BinaryOperator(
                    Box::new(Expression::Identifier("ki")),
                    OpCode::Asterisk,
                    Box::new(Expression::Identifier("kp")),
                )),
            )),
        );

        assert_eq!(ast, expected_ast);
    }
}
