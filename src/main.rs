#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::manual_assert)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::type_complexity)]
#![forbid(clippy::explicit_deref_methods)]
#![forbid(clippy::range_plus_one)]
#![forbid(clippy::semicolon_if_nothing_returned)]
#![forbid(clippy::map_unwrap_or)]
#![forbid(clippy::uninlined_format_args)]

use std::{io::Write as _, process};

use codegen::CodegenContext;
use error_handling::WLangError;
use lexer::{Lexer, LexerError};
use parser::Module;

use crate::{error_handling::Spanned, lexer::Token};

mod lexer;

mod codegen;

#[macro_use]
mod error_handling;

mod util;

mod parser;

/* Long term TODO list
 *  - Add function definition to errors
 *  - Add a "function already defined" error
 *  - Allow out-of-order and recursive functions
 *  - Add intrinsic attribute
 *  - Use function-based errors for parser
 */

fn main() {
    let wlang_src = std::fs::read_dir("wlang_src").unwrap();

    let context = inkwell::context::Context::create();
    let mut codegen_context = CodegenContext::new(&context);

    for file in wlang_src {
        let file = file.unwrap();
        let file_path = file.path();
        let source: &str = &String::from_utf8(std::fs::read(&file_path).unwrap()).unwrap();

        let file_name: String = file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();

        let ast = parse_file(&file_name, source);

        if let Err(err) = codegen_context.generate_module(&ast) {
            eprintln!("\n{}", err.render(source));
            process::exit(1);
        }
    }
}

fn parse_file<'a>(file_name: &str, file: &'a str) -> Module<'a> {
    let tokens: Result<Vec<Spanned<Token>>, LexerError> = Lexer::new(file).collect();

    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("\n{}", err.render(file));
            process::exit(1);
        }
    };

    let mut lex_file = std::fs::File::create(format!("./compiler_output/{file_name}.lex")).unwrap();
    write!(&mut lex_file, "{tokens:#?}").unwrap();

    let ast = parser::parse_module(&tokens);
    let ast: Module<'a> = match ast {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("\n{}", err.render(file));
            process::exit(1);
        }
    };

    let mut ast_file = std::fs::File::create(format!("./compiler_output/{file_name}.ast")).unwrap();
    write!(&mut ast_file, "{ast:#?}").unwrap();

    ast
}
