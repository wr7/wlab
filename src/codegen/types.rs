use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use wllvm::{builder::IntPredicate, debug_info::DIType, value::ValueEnum, Builder};
use wutil::Span;

use crate::{
    codegen::{self, codegen_unit::CodegenUnit, error, CodegenContext},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, OpCode},
};

use super::namestore::FieldInfo;

#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Type {
    i(u32),
    str,
    unit,
    bool,
    Struct { path: String },
}

#[derive(Clone)]
pub struct TypedValue<'ctx> {
    pub val: wllvm::Value<'ctx>,
    pub type_: Type,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str: Cow<'_, str> = match self {
            Type::i(n) => format!("i{n}").into(),
            Type::str => "str".into(),
            Type::unit => "()".into(),
            Type::bool => "bool".into(),
            Type::Struct { path } => Cow::Borrowed(path),
        };

        f.write_str(&str)
    }
}

impl Type {
    pub fn new(cc: &CodegenContext, type_: &S<ast::Path>) -> Result<Self, Diagnostic> {
        Ok(match &***type_ {
            [S("str", _)] => Self::str,
            [S("()", _)] => Self::unit,
            [S("bool", _)] => Self::bool,
            [type_] => {
                if let Some(num) = type_.strip_prefix("i").and_then(|n| n.parse::<u32>().ok()) {
                    Self::i(num)
                } else {
                    return Err(codegen::error::undefined_type(*type_));
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

    pub fn get_llvm_type<'ctx>(&self, context: &CodegenContext<'ctx>) -> wllvm::Type<'ctx> {
        match *self {
            Type::i(n) => context.context.int_type(n).into(),
            Type::str => context.core_types.str.into(),
            Type::unit => context.core_types.unit.into(),
            Type::bool => context.core_types.bool.into(),
            Type::Struct { ref path } => {
                let struct_info = context
                    .name_store
                    .get_item_from_string(path)
                    .as_struct()
                    .unwrap();

                let fields = struct_info
                    .fields
                    .iter()
                    .map(|FieldInfo { ty, .. }| ty.get_llvm_type(context))
                    .collect::<Vec<_>>();

                *context.context.struct_type(&fields, struct_info.packed)
            }
        }
    }

    pub fn get_dwarf_type<'ctx>(&self, cu: &CodegenUnit<'_, 'ctx>) -> DIType<'ctx> {
        cu.debug_context.get_type(self, cu)
    }
}

impl<'ctx> TypedValue<'ctx> {
    pub fn generate_operation(
        &self,
        builder: &Builder<'ctx>,
        lhs_span: Span,
        opcode: OpCode,
        rhs: &S<Self>,
    ) -> Result<Self, Diagnostic> {
        match self.type_ {
            Type::i(n) => self.generate_operation_int(n, builder, lhs_span, opcode, rhs),
            Type::unit | Type::str | Type::Struct { .. } => Err(
                codegen::error::undefined_operator(opcode, lhs_span, &self.type_),
            ),
            Type::bool => {
                if rhs.type_ != Type::bool {
                    return Err(codegen::error::unexpected_type(
                        rhs.1,
                        &Type::bool,
                        &rhs.type_,
                    ));
                }

                let Some((ValueEnum::IntValue(lhs), ValueEnum::IntValue(rhs))) =
                    self.val.downcast().zip(rhs.val.downcast())
                else {
                    unreachable!();
                };

                let val = match opcode {
                    OpCode::Or => builder.build_or(lhs, rhs, c""),
                    OpCode::And => builder.build_and(lhs, rhs, c""),
                    OpCode::NotEqual => builder.build_xor(lhs, rhs, c""),
                    OpCode::Equal => {
                        let xor = builder.build_xor(lhs, rhs, c"");
                        builder.build_not(xor, c"")
                    }
                    _ => {
                        return Err(codegen::error::undefined_operator(
                            opcode,
                            lhs_span,
                            &self.type_,
                        ))
                    }
                };

                Ok(Self {
                    type_: Type::bool,
                    val: val.into(),
                })
            }
        }
    }

    fn generate_operation_int(
        &self,
        bits: u32,
        builder: &Builder<'ctx>,
        lhs_span: Span,
        opcode: OpCode,
        rhs: &S<TypedValue<'ctx>>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        if rhs.type_ != Type::i(bits) {
            return Err(codegen::error::unexpected_type(
                rhs.1,
                &Type::i(bits),
                &rhs.type_,
            ));
        }

        let Some((ValueEnum::IntValue(lhs), ValueEnum::IntValue(rhs))) =
            self.val.downcast().zip(rhs.val.downcast())
        else {
            unreachable!();
        };

        let val;
        let type_;

        match opcode {
            OpCode::Plus => {
                val = builder.build_add(lhs, rhs, c"");
                type_ = Type::i(bits);
            }
            OpCode::Minus => {
                val = builder.build_sub(lhs, rhs, c"");
                type_ = Type::i(bits);
            }
            OpCode::Asterisk => {
                val = builder.build_mul(lhs, rhs, c"");
                type_ = Type::i(bits);
            }
            OpCode::Slash => {
                val = builder.build_sdiv(lhs, rhs, c"");
                type_ = Type::i(bits);
            }
            OpCode::Equal => {
                val = builder.build_icmp(IntPredicate::EQ, lhs, rhs, c"");
                type_ = Type::bool;
            }
            OpCode::NotEqual => {
                val = builder.build_icmp(IntPredicate::NE, lhs, rhs, c"");
                type_ = Type::bool;
            }
            OpCode::Greater => {
                val = builder.build_icmp(IntPredicate::SGT, lhs, rhs, c"");
                type_ = Type::bool;
            }
            OpCode::Less => {
                val = builder.build_icmp(IntPredicate::SLT, lhs, rhs, c"");
                type_ = Type::bool;
            }
            OpCode::GreaterEqual => {
                val = builder.build_icmp(IntPredicate::SGE, lhs, rhs, c"");
                type_ = Type::bool;
            }
            OpCode::LessEqual => {
                val = builder.build_icmp(IntPredicate::SLE, lhs, rhs, c"");
                type_ = Type::bool;
            }
            OpCode::And | OpCode::Or => {
                return Err(codegen::error::undefined_operator(
                    opcode,
                    lhs_span,
                    &self.type_,
                ))
            }
        };

        Ok(Self {
            type_,
            val: val.into(),
        })
    }
}
