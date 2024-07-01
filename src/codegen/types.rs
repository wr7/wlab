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
    unit,
    bool,
}

#[derive(Clone)]
pub struct TypedValue<'ctx> {
    pub val: BasicValueEnum<'ctx>,
    pub type_: Type,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Type::i32 => "i32",
            Type::str => "str",
            Type::unit => "()",
            Type::bool => "bool",
        };

        write!(f, "{str}")
    }
}

impl Type {
    pub fn new(type_: S<&str>) -> Result<Self, Diagnostic> {
        Ok(match *type_ {
            "i32" => Self::i32,
            "str" => Self::str,
            "()" => Self::unit,
            "bool" => Self::bool,
            _ => return Err(codegen::error::undefined_type(type_)),
        })
    }

    pub fn get_llvm_type<'ctx>(&self, generator: &CodegenUnit<'ctx>) -> BasicTypeEnum<'ctx> {
        match self {
            Type::i32 => generator.core_types.i32.into(),
            Type::str => generator.core_types.str.into(),
            Type::unit => generator.core_types.unit.into(),
            Type::bool => generator.core_types.bool.into(),
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
            Type::i32 => self.generate_operation_i32(builder, lhs_span, opcode, rhs),
            Type::unit | Type::str => Err(codegen::error::undefined_operator(
                opcode,
                lhs_span,
                &self.type_,
            )),
            Type::bool => {
                if rhs.type_ != Type::bool {
                    return Err(codegen::error::unexpected_type(
                        rhs.1,
                        &Type::bool,
                        &rhs.type_,
                    ));
                }

                let (BasicValueEnum::IntValue(lhs), BasicValueEnum::IntValue(rhs)) =
                    (self.val, rhs.val)
                else {
                    unreachable!();
                };

                let val = match opcode {
                    OpCode::Or => builder.build_or(lhs, rhs, ""),
                    OpCode::And => builder.build_and(lhs, rhs, ""),
                    OpCode::NotEqual => builder.build_xor(lhs, rhs, ""),
                    OpCode::Equal => {
                        let xor = builder.build_xor(lhs, rhs, "").unwrap();
                        builder.build_not(xor, "")
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
                    val: val.unwrap().into(),
                })
            }
        }
    }

    fn generate_operation_i32(
        &self,
        builder: &Builder<'ctx>,
        lhs_span: Span,
        opcode: OpCode,
        rhs: &S<TypedValue<'ctx>>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        if rhs.type_ != Type::i32 {
            return Err(codegen::error::unexpected_type(
                rhs.1,
                &Type::i32,
                &rhs.type_,
            ));
        }

        let (BasicValueEnum::IntValue(lhs), BasicValueEnum::IntValue(rhs)) = (self.val, rhs.val)
        else {
            unreachable!();
        };

        let val;
        let type_;

        match opcode {
            OpCode::Plus => {
                val = builder.build_int_add(lhs, rhs, "");
                type_ = Type::i32;
            }
            OpCode::Minus => {
                val = builder.build_int_sub(lhs, rhs, "");
                type_ = Type::i32;
            }
            OpCode::Asterisk => {
                val = builder.build_int_mul(lhs, rhs, "");
                type_ = Type::i32;
            }
            OpCode::Slash => {
                val = builder.build_int_signed_div(lhs, rhs, "");
                type_ = Type::i32;
            }
            OpCode::Equal => {
                val = builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "");
                type_ = Type::bool;
            }
            OpCode::NotEqual => {
                val = builder.build_int_compare(inkwell::IntPredicate::NE, lhs, rhs, "");
                type_ = Type::bool;
            }
            OpCode::Greater => {
                val = builder.build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "");
                type_ = Type::bool;
            }
            OpCode::Less => {
                val = builder.build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, "");
                type_ = Type::bool;
            }
            OpCode::GreaterEqual => {
                val = builder.build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "");
                type_ = Type::bool;
            }
            OpCode::LessEqual => {
                val = builder.build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "");
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
            val: val.unwrap().into(),
        })
    }
}
