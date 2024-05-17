use std::process::abort;

use crate::{
    lexer::{BracketType, Token},
    T,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Item<'a> {
    Identifier(&'a str),
    BinaryOperator(Box<Self>, OpCode, Box<Self>),
    Function(&'a str, Vec<Item<'a>>),
    CompoundExpression(Vec<Self>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Plus,
    Minus,
    Asterisk,
    Slash,
}

pub fn try_parse_ident<'a>(tokens: &'a [Token<'a>]) -> Option<Item<'a>> {
    if let Token::Identifier(ident) = tokens.first()? {
        if tokens.len() == 1 {
            return Some(Item::Identifier(ident));
        }
    }

    None
}

pub fn try_parse_bracket_expr<'a>(tokens: &'a [Token<'a>]) -> Option<Item<'a>> {
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
                    return Some(Item::CompoundExpression(parse_expr_list(&tokens[1..i])));
                } else {
                    return try_parse_expr(&tokens[1..i]);
                }
            }
        }
    }

    dbg!(tokens);
    panic!("Expected closing bracket, reached end of expression")
}

pub fn try_parse_function<'a>(tokens: &'a [Token<'a>]) -> Option<Item<'a>> {
    let mut token_iter = tokens.iter().enumerate();

    let [&T!("fn"), Token::Identifier(fn_name)] = [token_iter.next()?.1, token_iter.next()?.1]
    else {
        return None;
    };
    let [T!("("), T!(")")] = [token_iter.next()?.1, token_iter.next()?.1] else {
        panic!("Expected parenthesis");
    };

    let (body_start, _) = token_iter.next().unwrap();

    let Item::CompoundExpression(body) = try_parse_bracket_expr(&tokens[body_start..]).unwrap()
    else {
        unreachable!()
    };

    Some(Item::Function(&fn_name, body))
}

pub fn parse_expr_list<'a>(tokens: &'a [Token<'a>]) -> Vec<Item<'a>> {
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
            items.push(try_parse_expr(&tokens[statement_start..i]).unwrap());
        } else if tok == &T!("}") {
            items.push(try_parse_expr(&tokens[statement_start..=i]).unwrap());
        }

        statement_start = i + 1;
    }

    if statement_start != tokens.len() {
        items.push(try_parse_expr(&tokens[statement_start..]).unwrap());
    }

    items
}

pub fn try_parse_bin<'a>(
    tokens: &'a [Token<'a>],
    opcodes: &[(Token<'a>, OpCode)],
) -> Option<Item<'a>> {
    let mut bracket_level = 0;

    for (i, tok) in tokens.iter().enumerate().rev() {
        if matches!(tok, Token::OpenBracket(_)) {
            if bracket_level == 0 {
                abort()
            }
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
                return Some(Item::BinaryOperator(
                    Box::new(try_parse_expr(&tokens[0..i]).unwrap()),
                    *opcode,
                    Box::new(try_parse_expr(&tokens[(i + 1)..]).unwrap()),
                ));
            }
        }
    }

    return None;
}

pub fn try_parse_expr<'a>(tokens: &'a [Token<'a>]) -> Option<Item<'a>> {
    if tokens.len() == 0 {
        return None;
    }

    let rules = [
        |tokens| try_parse_function(tokens),
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
        let expected_ast = Item::BinaryOperator(
            Box::new(Item::BinaryOperator(
                Box::new(Item::BinaryOperator(
                    Box::new(Item::Identifier("foo")),
                    OpCode::Plus,
                    Box::new(Item::BinaryOperator(
                        Box::new(Item::BinaryOperator(
                            Box::new(Item::Identifier("bar")),
                            OpCode::Slash,
                            Box::new(Item::Identifier("biz")),
                        )),
                        OpCode::Asterisk,
                        Box::new(Item::BinaryOperator(
                            Box::new(Item::Identifier("bang")),
                            OpCode::Minus,
                            Box::new(Item::Identifier("zig")),
                        )),
                    )),
                )),
                OpCode::Plus,
                Box::new(Item::BinaryOperator(
                    Box::new(Item::Identifier("his")),
                    OpCode::Asterisk,
                    Box::new(Item::Identifier("fan")),
                )),
            )),
            OpCode::Plus,
            Box::new(Item::BinaryOperator(
                Box::new(Item::Identifier("se")),
                OpCode::Slash,
                Box::new(Item::BinaryOperator(
                    Box::new(Item::Identifier("ki")),
                    OpCode::Asterisk,
                    Box::new(Item::Identifier("kp")),
                )),
            )),
        );

        assert_eq!(ast, expected_ast);
    }
}
