use test::TestArguments;

pub mod test;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum SubcommandArguments {
    Test(test::TestArguments),
    Bless,
}

#[derive(Clone, Debug)]
pub struct Arguments {
    pub sub_args: SubcommandArguments,
}

impl Arguments {
    pub const HELP: &'static str = ::core::concat!(
        "wtool - commandline tool for managing the wlab compiler\n",
        "\n",
        "Usage:\n",
        "  wtool help [subcommand]>\n",
        "  wtool test <FLAGS> [--test=/path/to/test]\n",
        "  wtool bless <FLAGS> (--all | --test=/path/to/test)\n",
    );

    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1);

        match args.next().as_deref() {
            Some("help" | "-h" | "--help") => {
                let help_msg = match args.next().as_deref() {
                    Some("test") => TestArguments::HELP,
                    Some("bless") => todo!(),
                    None => Self::HELP,
                    Some(subcommand) => {
                        eprintln!(
                            "\x1b[0;31;1mwtool help:\x1b[m invalid subcommand \x1b[m`\x1b[1m{subcommand}\x1b[m`"
                        );

                        std::process::exit(1);
                    }
                };
                eprint!("{help_msg}");
                std::process::exit(0);
            }
            Some("test") => Self {
                sub_args: SubcommandArguments::Test(
                    test::TestArguments::parse(args).unwrap_or_else(|err| {
                        eprint!("\x1b[0;31;1mwtool-test:\x1b[m ");
                        eprintln!("{err}");
                        std::process::exit(1)
                    }),
                ),
            },
            Some("bless") => todo!(),
            Some(subcommand) => {
                eprintln!(
                    "\x1b[0;31;1mwtool:\x1b[m invalid subcommand \x1b[m`\x1b[1m{subcommand}\x1b[m`"
                );
                std::process::exit(1);
            }
            None => {
                print!("{}", Self::HELP);
                std::process::exit(0);
            }
        }
    }
}
