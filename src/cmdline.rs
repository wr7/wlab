use std::borrow::Cow;

use argtea::argtea_impl;
use wllvm::target::OptLevel;

pub struct Parameters {
    pub input_files: Vec<String>,
    pub out_dir: Cow<'static, str>,
    pub opt_level: OptLevel,
    pub lex_files: bool,
    pub generate_ast: bool,
    pub generate_ir: bool,
    pub generate_asm: bool,
    pub generate_object: bool,
}

argtea_impl! {
    {
        /// Prints flags, usage, and descriptions.
        ("--help" | "-h") => {
            eprint!("{}", Self::HELP);

            std::process::exit(0);
        }

        /// Sets the output directory to write the generated files to.
        ("--output-directory" | "--output-dir" | "--output" | "-o", output_dir) => {
            out_dir = Cow::Owned(
                output_dir.ok_or("expected output directory")?
            );
        }

        /// Disables all compiler optimization
        ("-O0") => {
            opt_level = OptLevel::None;
        }

        /// Sets optimization level to 1.
        ("-O1") => {
            opt_level = OptLevel::Less;
        }

        /// Sets optimization level to 2 (default).
        ("-O2") => {
            opt_level = OptLevel::Default;
        }

        /// Sets optimization level to 3 (maximum).
        ("-O3") => {
            opt_level = OptLevel::Aggressive;
        }

        /// Generates a `.lex` file for each input.
        ///
        /// Unlike the other options, the generated file will be of the form
        /// `file_name.lex` where `file_name` is the input file name with the `.wlang`
        /// extention removed if present.
        ("--lex" | "-l") => {
            lex_files = true;
        }

        /// Don't generate a `.lex` file for each input (default).
        ("--dont-lex" | "--no-lex") => {
            lex_files = false;
        }

        /// Generates a `.ast` (Abstract Syntax Tree) file for each input.
        ///
        /// Unlike the other options, the generated file will be of the form
        /// `file_name.ast` where `file_name` is the input file name with the `.wlang`
        /// extention removed if present.
        ("--ast" | "--parse" | "-a") => {
            generate_ast = true;
        }

        /// Don't generate a `.ast` (Abstract Syntax Tree) file for each input
        /// (default).
        ("--no-ast" | "--dont-parse") => {
            generate_ast = false;
        }

        /// Generates a `.ll` LLVM IR file for each input.
        ("--llvm-ir" | "--ir" | "-i") => {
            generate_ir = true;
        }

        /// Dont generate a `.ll` LLVM IR file for each input (default).
        ("--no-llvm-ir" | "--no-ir") => {
            generate_ir = false;
        }

        /// Generate a `.asm` assembly file for each input.
        ("--assembly" | "--asm" | "-S") => {
            generate_asm = true;
        }

        /// Don't generate a `.asm` assembly file for each input (default).
        ("--no-assembly" | "--no-asm") => {
            generate_asm = false;
        }

        /// Generate a `.o` object file for each input (default).
        ("--object" | "-s") => {
            generate_object = true;
        }

        /// Don't generate a `.o` object file for each input.
        ("--no-object") => {
            generate_object = false;
        }

        /// Input files
        (input_file) => {
            if input_file.starts_with("-") {
                return Err(format!("invalid flag `\x1b[1m{input_file}\x1b[m`").into());
            }
            input_files.push(input_file);
        }
    }

    impl Parameters {
        const HELP: &'static str = argtea::simple_format!(
            "wlab compiler"
            ""
            "Usage: wlab [OPTION]... [INPUT_FILE]..."
            ""
            "Options:"
            docs!()
        );

        fn parse() -> Result<Self, Cow<'static, str>> {
            let mut input_files: Vec<String> = Vec::new();
            let mut out_dir: Cow<'static, str> = Cow::Borrowed("./");
            let mut opt_level: OptLevel = OptLevel::Default;
            let mut lex_files: bool = false;
            let mut generate_ast: bool = false;
            let mut generate_ir: bool = false;
            let mut generate_asm: bool = false;
            let mut generate_object: bool = true;

            let mut args = std::env::args().skip(1);

            parse!(args);

            return Ok(Self { input_files, out_dir, opt_level, lex_files, generate_ast, generate_ir, generate_asm, generate_object });
        }
    }
}
