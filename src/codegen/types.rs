use std::fmt::Display;

use inkwell::{builder::Builder, types::BasicTypeEnum, values::BasicValueEnum};
use wutil::Span;

use crate::{
    codegen::{self, CodegenUnit},
    error_handling::{Diagnostic, Spanned as S},
    parser::OpCode,
};

#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Type {
    i32,
    str,
}

#[derive(Clone)]
pub struct TypedValue<'ctx> {
    pub val: BasicValueEnum<'ctx>,
    pub type_: Type,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::i32 => write!(f, "i32"),
            Type::str => write!(f, "str"),
        }
    }
}

impl Type {
    pub fn new(type_: &str, span: Span) -> Result<Self, Diagnostic> {
        Ok(match type_ {
            "i32" => Self::i32,
            "str" => Self::str,
            _ => return Err(codegen::error::undefined_type(S(type_, span))),
        })
    }

    pub fn get_llvm_type<'ctx>(&self, generator: &CodegenUnit<'ctx>) -> BasicTypeEnum<'ctx> {
        match self {
            Type::i32 => generator.core_types.i32.into(),
            Type::str => generator.core_types.str.into(),
        }
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
            Type::i32 => {
                if rhs.type_ != Type::i32 {
                    return Err(codegen::error::unexpected_type(
                        rhs.1,
                        &Type::i32,
                        &rhs.type_,
                    ));
                }

                let (BasicValueEnum::IntValue(lhs), BasicValueEnum::IntValue(rhs)) =
                    (self.val, rhs.val)
                else {
                    unreachable!();
                };

                let val = match opcode {
                    OpCode::Plus => builder.build_int_add(lhs, rhs, ""),
                    OpCode::Minus => builder.build_int_sub(lhs, rhs, ""),
                    OpCode::Asterisk => builder.build_int_mul(lhs, rhs, ""),
                    OpCode::Slash => builder.build_int_signed_div(lhs, rhs, ""),
                };

                Ok(Self {
                    type_: Type::i32,
                    val: val.unwrap().into(),
                })
            }
            Type::str => Err(codegen::error::undefined_operator(
                opcode,
                lhs_span,
                &self.type_,
            )),
        }
    }
}
