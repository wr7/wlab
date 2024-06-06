use std::fmt::Display;

use inkwell::{builder::Builder, context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::parser::OpCode;

use super::error::CodegenError;

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    i32,
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
        }
    }
}

impl Type {
    pub fn new<'a>(type_: &'a str) -> Result<Self, CodegenError<'a>> {
        Ok(match type_ {
            "i32" => Self::i32,
            _ => return Err(CodegenError::UndefinedType(type_)),
        })
    }

    pub fn get_llvm_type<'ctx>(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            Type::i32 => context.i32_type().into(),
        }
    }
}

impl<'ctx> TypedValue<'ctx> {
    pub fn generate_operation(
        &self,
        builder: &Builder<'ctx>,
        opcode: OpCode,
        rhs: Self,
    ) -> Result<Self, CodegenError<'static>> {
        match self.type_ {
            Type::i32 => {
                if rhs.type_ != Type::i32 {
                    // Incorrect type
                    todo!() // rhs span is required for error. Parser support is needed
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
        }
    }
}
