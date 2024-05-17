use lexer::{Lexer, LexerError};

use crate::lexer::{BracketType, Token};

mod lexer;

mod parser;

fn main() {
    let tokens: Result<Vec<Token>, LexerError> =
        Lexer::new("foo + bar/biz*(bang-zig)+(his*fan)/four+se/(ki*kp)").collect();
    let tokens: Vec<Token> = tokens.unwrap();

    dbg!(parser::try_parse_expr(&tokens));
}
