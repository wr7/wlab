use std::iter::Skip;

use argtea::argtea_impl;

#[derive(Clone, Debug)]
pub enum TestTarget {
    All,
    Tests(Vec<String>),
}

impl TestTarget {
    pub fn add_test(&mut self, test: String) {
        match self {
            TestTarget::All => *self = Self::Tests(vec![test]),
            TestTarget::Tests(tests) => tests.push(test),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TestArguments {
    pub target: TestTarget,
}

argtea_impl! {
    {
        /// Displays typical usage and all flags.
        ("--help" | "-h") => {
            print!("{}", Self::HELP);
            std::process::exit(0);
        }

        /// Adds a test to the list of tests to run.
        ("--test" | "-t", test) => {
            let test = test.ok_or_else(|| "expected test")?;

            target.add_test(test);
        }

        /// Runs all tests.
        ("--all") => {
            target = TestTarget::All;
        }

        #[hidden]
        (invalid_flag) => {
            Err(format!("invalid flag \x1b[m`\x1b[1m{invalid_flag}\x1b[m`"))?;
        }
    }
    impl TestArguments {
        const HELP: &'static str = argtea::simple_format!(
            "wtool test: runs `wlab` tests"
            ""
            "Usage:"
            "  wtool test [FLAGS] [--test=</path/to/test>..]"
            ""
            "If no tests are specified, all tests are checked"
            ""
            "Options:"
            docs!()
        );

        fn parse(mut args: Skip<std::env::Args>) -> Result<Self, String> {
            let mut target = TestTarget::All;

            parse!(args);

            return Ok(Self {
                target,
            });
        }
    }
}
