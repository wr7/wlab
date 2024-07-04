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

use error_handling::WLangError;
use lexer::{Lexer, LexerError};
use parser::Module;
use util::MemoryStore;

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

    let mut asts = Vec::new();
    let mut src_files = Vec::new();

    let src_store = MemoryStore::new();
    let tok_store = MemoryStore::new(); // TODO fix parser lifetime hell and remove lex store

    for file in wlang_src {
        let file = file.unwrap();
        let file_path = file.path();
        let source: &str =
            src_store.add(String::from_utf8(std::fs::read(&file_path).unwrap()).unwrap());

        let file_name: String = file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();

        asts.push(parse_file(&tok_store, &file_name, source));
        src_files.push(source);
    }

    if let Err((file_no, err)) = codegen::generate_code(&asts) {
        eprintln!("\n{}", err.render(src_files[file_no]));
        process::exit(1);
    };
}

// TODO fix parser lifetime hell and remove lex store
fn parse_file<'s, 'a: 's>(
    lex_store: &'s MemoryStore<Vec<Spanned<Token<'a>>>>,
    file_name: &str,
    file: &'a str,
) -> Module<'s> {
    let tokens: Result<Vec<Spanned<Token>>, LexerError> = Lexer::new(file).collect();

    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("\n{}", err.render(file));
            process::exit(1);
        }
    };

    let tokens = &*lex_store.add(tokens);

    let mut lex_file = std::fs::File::create(format!("./compiler_output/{file_name}.lex")).unwrap();
    write!(&mut lex_file, "{tokens:#?}").unwrap();

    let ast = parser::parse_module(&tokens);
    let ast: Module<'s> = match ast {
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
