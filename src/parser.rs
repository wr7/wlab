use crate::lexer::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Item<'a> {
    Identifier(&'a str),
    BinaryOperator(Box<Self>, OpCode, Box<Self>),
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
                return try_parse_expr(&tokens[1..i]);
            }
        }
    }

    dbg!(tokens);
    panic!("Expected closing bracket, reached end of expression")
}

pub fn try_parse_bin<'a>(
    tokens: &'a [Token<'a>],
    opcodes: &[(Token<'a>, OpCode)],
) -> Option<Item<'a>> {
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
        |tokens| try_parse_ident(tokens),
        |tokens| try_parse_bracket_expr(tokens),
        |tokens| {
            try_parse_bin(
                tokens,
                &[(Token::Plus, OpCode::Plus), (Token::Minus, OpCode::Minus)],
            )
        },
        |tokens| {
            try_parse_bin(
                tokens,
                &[
                    (Token::Asterisk, OpCode::Asterisk),
                    (Token::Slash, OpCode::Slash),
                ],
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
