use std::fmt::Display;

use inkwell::{builder::Builder, types::BasicTypeEnum, values::BasicValueEnum};
use wutil::Span;

use crate::{error_handling::Spanned, parser::OpCode};

use super::{error::CodegenError, CodegenUnit};

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
    pub fn new<'a>(type_: &'a str) -> Result<Self, CodegenError<'a>> {
        Ok(match type_ {
            "i32" => Self::i32,
            "str" => Self::str,
            _ => return Err(CodegenError::UndefinedType(type_)),
        })
    }

    pub fn get_llvm_type<'ctx>(&self, generator: &CodegenUnit<'ctx>) -> BasicTypeEnum<'ctx> {
        match self {
            Type::i32 => generator.core_types.i32.clone().into(),
            Type::str => generator.core_types.str.clone().into(),
        }
    }
}

impl<'ctx> TypedValue<'ctx> {
    pub fn generate_operation(
        &self,
        builder: &Builder<'ctx>,
        lhs_span: Span,
        opcode: OpCode,
        rhs: Spanned<Self>,
    ) -> Result<Self, CodegenError<'static>> {
        match self.type_ {
            Type::i32 => {
                if rhs.type_ != Type::i32 {
                    return Err(CodegenError::UnexpectedType(
                        rhs.1,
                        "i32".into(),
                        rhs.type_.to_string(),
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
            Type::str => Err(CodegenError::UndefinedOperator(
                opcode,
                lhs_span,
                self.type_.to_string(),
            )),
        }
    }
}
