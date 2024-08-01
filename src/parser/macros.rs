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
            $(let $binding =)? $crate::parser::macros::generate_directive!{_remaining $directive_name $params $(else $else)?};
        )*

        $($(let $remaining_toks = _remaining;)?)?

        $body
    }};
}

macro_rules! generate_else {
    {} => {None};
    {$else:block} => {$else};
    {$fallback_else:block $else:block} => {$else};
}

macro_rules! generate_directive {
    // do(?|?remaining_tokens| expr)
    // Evaluates a rust expression for parsing.
    // The remaining tokens can be accessed with the captured identifier
    {
            $tokens:ident
        do(|$($capture:ident)?| $block:expr)
    } => {{
        $(let $capture = &mut $tokens;)?
        $block
    }};
    {
            $tokens:ident
        do(|$($capture:ident)?| $block:expr)
        else $else:block
    } => {compile_error!("`do` directive cannot have an else block. Instead, wrap in `required` or `expect`.")};
    {
            $tokens:ident
        do($block:expr)
        else $else:block
    } => {compile_error!("`do` directive cannot have an else block. Instead, wrap in `required` or `expect`.")};


    // `token(?"token_literal")`
    // Takes a (specific) token or yields `None`
    {
            $tokens:ident
        token($($tok:tt)?)
        $(else $else:block)?
    } => {
        if let [token $(@ S(T!($tok), _))?, _tmp_remaining @ ..] = $tokens {
            #[allow(unused_assignments)]
            { $tokens = _tmp_remaining; }
            Some(token)
        } else {
            $crate::parser::macros::generate_else!{{None} $($else)?}
        }
    };

    // `ident()`
    // Takes an identifier token
    {
            $tokens:ident
        ident()
        $(else $else:block)?
    } => {{
        if let Some((_tmp_tok @ S(Token::Identifier(_tmp_ident), _), _tmp_remaining)) = $tokens.split_first() {
            #[allow(unused_assignments)]
            { $tokens = _tmp_remaining; }
            Some((*_tmp_ident, _tmp_tok))
        } else {
            $crate::parser::macros::generate_else!{$($else)?}
        }
    }};

    // `bracketed(*directives)`
    // Takes a bracketed pattern and returns it as a tuple
    {
            $tokens:ident
        bracketed(
            BracketType::$type:ident:
            {$(
                $directive_name:ident $params:tt
                $(else $inner_else:block)?
            );* $(;)?}
        )
        $(else $else:block)?
    } => {{
        let mut nb_iter = $crate::parser::util::NonBracketedIter::new($tokens);

        if let Some(_tmp_opening @ S(Token::OpenBracket($crate::lexer::BracketType::$type), _)) = nb_iter.next() {
            let Some(_tmp_closing @ S(Token::CloseBracket($crate::lexer::BracketType::$type), _)) = nb_iter.next() else {
                unreachable!()
            };
            let closing_idx = $crate::util::SliceExt::elem_offset($tokens, _tmp_closing).unwrap();

            let val = {
                let mut _remaining = &$tokens[1..closing_idx];

                (
                    _tmp_opening,
                    $(
                        $crate::parser::macros::generate_directive!{
                            _remaining $directive_name $params
                            $(else $inner_else)?
                        },
                    )*
                    _tmp_closing
                )
            };

            $tokens = &$tokens[closing_idx + 1..];

            Some(val)
        } else {
            $crate::parser::macros::generate_else!{$($else)?}
        }
    }};

    // required(directive)
    // Unwraps and returns a directive
    {
            $tokens:ident
        required(
            $directive_name:ident $params:tt
            $(else $inner_else:block)?
        )
        $(else $else:block)?
    } => {
        if let Some(val) = $crate::parser::macros::generate_directive!{
            $tokens $directive_name $params
            $(else $inner_else)?
        } {
            val
        } else {
            $crate::parser::macros::generate_else!{{return Ok(None)} $($else)?}
        }
    };

    // required{+directives}
    // Unwraps a list of directions and assigns them to bindings.
    {
            $tokens:ident
        required {$(
            $directive_name:ident $params:tt
            $(else $inner_else:block)?
            $(@ $binding:tt)?
        );+ $(;)?}
    } => {
        $($(let $binding =)? $crate::parser::macros::generate_directive!{
                $tokens
            required(
                $directive_name $params
                $(else $inner_else)?
            )
        };)+
    };
    {
            $tokens:ident
        required {$(
            $directive_name:ident $params:tt
            $(else $inner_else:block)?
            $(@ $binding:tt)?
        );+ $(;)?}
        $(else $else:block)?
    } => {compile_error!("curly bracked directive `required` cannot have an else block")};

    // expect(directive)
    // Executes an `else` branch if a directive returns `None`.
    // This is useful for directives like `do` which do not have else branches
    // because they may not return `Option`s
    {
            $tokens:ident
        expect(
            $directive_name:ident $params:tt
            $(else $inner_else:block)?
        )
        $(else $else:block)?
    } => {
        if let Some(val) = $crate::parser::macros::generate_directive!{
            $tokens $directive_name $params
            $(else $inner_else)?
        } {
            Some(val)
        } else {
            $crate::parser::macros::generate_else!{$($else)?}
        }
    };

    // all(+directives)
    // Unwraps a list of directives and returns them as an optional tuple.
    //
    // If a single directive fails to match, the following ones are not evaluated.
    // Additionally, the token stream is returned to its original state
    //
    // NOTE: any inner bindings made are local to inside of the directive.
    // these may be used inside of `do` directives and else blocks
    {
            $tokens:ident
        all ($(
            $directive_name:ident $params:tt
            $(else $inner_else:block)?
            $(@ $binding:ident)?
        );+ $(;)?)
        $(else $else:block)?
    } => {{
        let mut _remaining = $tokens;

        $($(let $binding;)?)+

        if let Some(_tmp_val) =
            loop {
                break Some(($(
                    if let Some(_tmp_val) = $crate::parser::macros::generate_directive!{
                            _remaining
                        $directive_name $params
                        $(else $inner_else)?
                    } {$($binding = _tmp_val;)? _tmp_val} else {
                        break None;
                    }
                ),+));
            }
         {
            $tokens = _remaining;
            Some(_tmp_val)
        } else {
            $crate::parser::macros::generate_else! {$($else)?}
        }
    }};
}

pub(super) use generate_directive;
pub(super) use generate_else;
pub(super) use match_tokens;
