use std::process;

pub use args_macro::Parameters;

#[allow(unreachable_code)]
mod args_macro {
    use std::borrow::Cow;

    use wllvm::target::OptLevel;

    argwerk::define! {
        /// wlab compiler
        #[usage = "wlab [OPTION]... [INPUT_FILE]..."]
        pub struct Parameters {
            pub input_files: Vec<String>,
            pub out_dir: Cow<'static, str> = Cow::Borrowed("./"),
            pub opt_level: OptLevel,
            pub lex_files: bool = false,
            pub generate_ast: bool = false,
            pub generate_ir: bool = false,
            pub generate_asm: bool = false,
            pub generate_object: bool = true,
        }

        /// Prints flags, usage, and descriptions.
        ["--help" | "-h"] => {
            Self::print_help()
        }

        /// Sets the output directory to write the generated files to.
        ["--output-directory" | "--output" | "--out" | "-o", output_dir] => {
            out_dir = Cow::Owned(output_dir);
        }

        /// Disables all compiler optimization
        ["-O0"] => {
            opt_level = OptLevel::None;
        }

        /// Sets optimization level to 1.
        ["-O1"] => {
            opt_level = OptLevel::Less;
        }

        /// Sets optimization level to 2 (default).
        ["-O2"] => {
            opt_level = OptLevel::Default;
        }

        /// Sets optimization level to 3 (maximum).
        ["-O3"] => {
            opt_level = OptLevel::Aggressive;
        }

        /// Generates a `.lex` file for each input.
        ///
        ///
        /// Unlike the other options, the generated file will be of the form
        /// `file_name.lex` where `file_name` is the input file name with the
        /// `.wlang` extention removed if present.
        ["--lex" | "-l"] => {
            lex_files = true;
        }

        /// Don't generate a `.lex` file for each input (default).
        ["--dont-lex" | "--no-lex" | "-nl"] => {
            lex_files = false;
        }

        /// Generates a `.ast` (Abstract Syntax Tree) file for each input.
        ///
        ///
        /// Unlike the other options, the generated file will be of the form
        /// `file_name.ast` where `file_name` is the input file name with the
        /// `.wlang` extention removed if present.
        ["--ast" | "--parse" | "-a"] => {
            generate_ast = true;
        }

        /// Don't generate a `.ast` (Abstract Syntax Tree) file for each input
        /// (default).
        ["--no-ast" | "--dont-parse" | "-na"] => {
            generate_ast = false;
        }

        /// Generates a `.ll` LLVM IR file for each input.
        ["--llvm-ir" | "--ir" | "-i"] => {
            generate_ir = true;
        }

        /// Dont generate a `.ll` LLVM IR file for each input (default).
        ["--no-llvm-ir" | "--no-ir" | "-ni"] => {
            generate_ir = false;
        }

        /// Generate a `.asm` assembly file for each input.
        ["--assembly" | "--asm" | "-S"] => {
            generate_asm = true;
        }

        /// Don't generate a `.asm` assembly file for each input (default).
        ["--no-assembly" | "--no-asm" | "-nS"] => {
            generate_asm = false;
        }

        /// Generate a `.o` object file for each input (default).
        ["--object" | "-s"] => {
            generate_object = true;
        }

        /// Don't generate a `.o` object file for each input.
        ["--object" | "-s"] => {
            generate_object = false;
        }

        /// Input files
        [input_file] => {
            input_files.push(input_file);
        }
    }
}

/// Trims, concatenates, performs line wrapping, and indents doc comments.
fn format_doccoments(docs: &[&str], indent_level: usize) -> String {
    let mut retval = String::new();

    let mut chars = 0;
    for d in docs {
        let d = d.trim();

        if d.is_empty() {
            retval.push('\n');

            chars = 0;
            continue;
        }

        let mut iter = d.split_ascii_whitespace().peekable();

        while let Some(w) = iter.peek() {
            if chars == 0 {
                for _ in 0..indent_level {
                    retval.push(' ');
                }

                retval.push_str(w);
                chars = w.len() + indent_level;
                iter.next();
            } else if chars + w.len() < 80 {
                retval.push(' ');
                retval.push_str(w);
                chars += 1 + w.len();
                iter.next();
            } else {
                retval.push('\n');
                chars = 0;
            }
        }
    }

    retval
}

impl Parameters {
    fn print_help() -> ! {
        println!("Usage: {}\n", Self::HELP.usage);

        println!("{}\n", format_doccoments(Self::HELP.docs, 1));

        println!("Options:");
        for switch in Self::HELP.switches {
            println!("    {}", switch.usage);
            let docs = format_doccoments(switch.docs, 8);
            println!("{docs}\n\n");
        }

        process::exit(0);
    }
}
