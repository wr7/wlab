pub mod directives;

/// Matches a set of tokens and executes a corresponding block of code.
macro_rules! match_tokens {
    {
        $tokens_name:ident: {
            $(
                $directive_name:ident $params:tt
                $(else $else:block)?
                $(@ $binding:tt)?
            );*
            $(;)?
        } => $(|$($remaining_toks:ident)?|)? $body:block
    } => {{
        let mut _remaining = $tokens_name;

        $(
            $(let $binding =)? $crate::parser::macros::directives::$directive_name!{_remaining $params $(else $else)?};
        )*

        $($(let $remaining_toks = _remaining;)?)?

        $body
    }};
}

/// Helper macro for [`match_tokens`]
macro_rules! generate_else {
    {} => {None};
    {$else:block} => {$else};
    {$fallback_else:block $else:block} => {$else};
}

pub(super) use generate_else;
pub(super) use match_tokens;
