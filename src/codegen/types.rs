use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use wllvm::debug_info::DIType;

use crate::{
    codegen::{codegen_unit::CodegenUnit, error, CodegenContext},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast,
};

#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Type {
    i(u32),
    str,
    unit,
    never,
    bool,
    Struct { path: String },
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str: Cow<'_, str> = match self {
            Type::i(n) => format!("i{n}").into(),
            Type::str => "str".into(),
            Type::unit => "()".into(),
            Type::bool => "bool".into(),
            Type::Struct { path } => Cow::Borrowed(path),
            Type::never => "!".into(),
        };

        f.write_str(&str)
    }
}

impl Type {
    /// Checks if `self` can be used in-place-of `type_`
    ///
    /// This has special handling for the `!` type which can be used in-place-of any type.
    pub fn is(&self, type_: &Self) -> bool {
        self == type_ || self == &Type::never
    }

    pub fn new(
        cc: &CodegenContext,
        crate_name: &str,
        type_: &S<ast::Path>,
    ) -> Result<Self, Diagnostic> {
        Ok(match &***type_ {
            [S("str", _)] => Self::str,
            [S("()", _)] => Self::unit,
            [S("!", _)] => Self::never,
            [S("bool", _)] => Self::bool,
            [type_] => {
                if let Some(num) = type_.strip_prefix("i").and_then(|n| n.parse::<u32>().ok()) {
                    Self::i(num)
                } else {
                    cc.name_store
                        .get_item_in_crate(crate_name, *type_)?
                        .as_struct()
                        .ok_or_else(|| error::not_type(*type_))?;

                    return Ok(Self::Struct {
                        path: format!("{crate_name}::{}", **type_),
                    });
                }
            }
            _ => {
                let mut path = String::new();

                for (i, &segment) in type_.iter().enumerate() {
                    if i != 0 {
                        path.push_str("::");
                    }

                    path.push_str(*segment);
                }

                cc.name_store
                    .get_item(type_)?
                    .as_struct()
                    .ok_or_else(|| error::not_type(S(&path, type_.1)))?;

                Self::Struct { path }
            }
        })
    }

    /// Gets the underlying LLVM type or `None` if the type is uninstantiable
    pub fn llvm_type<'ctx>(&self, context: &CodegenContext<'ctx>) -> Option<wllvm::Type<'ctx>> {
        Some(match *self {
            Type::i(n) => context.context.int_type(n).into(),
            Type::str => context.core_types.str.into(),
            Type::unit => context.core_types.unit.into(),
            Type::bool => context.core_types.bool.into(),
            Type::Struct { ref path } => {
                let struct_info = context
                    .name_store
                    .get_item_from_string(path)
                    .unwrap()
                    .as_struct()
                    .unwrap();

                return struct_info.llvm_type.as_deref().copied();
            }
            Type::never => return None,
        })
    }

    pub fn get_dwarf_type<'ctx>(&self, cu: &CodegenUnit<'_, 'ctx>) -> DIType<'ctx> {
        cu.debug_context.get_type(self, cu)
    }
}
