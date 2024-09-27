use wutil::Span;

use crate::{
    diagnostic as d,
    error_handling::{Diagnostic, Hint},
};

pub fn unreachable_code(terminating_statement: Span, dead_code: Span) -> Diagnostic {
    d! {
        "Warning: unreachable code",
        [
            Hint::new_info("Because of this statement here,", terminating_statement),
            Hint::new_warning("This code cannot be reached", dead_code),
        ],
    }
}
