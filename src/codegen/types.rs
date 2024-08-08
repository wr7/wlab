use std::fmt::Display;

use wllvm::{builder::IntPredicate, debug_info::DIType, value::ValueEnum, Builder};
use wutil::Span;

use crate::{
    codegen::{self, codegen_unit::CodegenUnit, CodegenContext},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::OpCode,
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
    pub val: wllvm::Value<'ctx>,
    pub type_: Type,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

    pub fn get_llvm_type<'ctx>(&self, context: &CodegenContext<'ctx>) -> wllvm::Type<'ctx> {
        match self {
            Type::i32 => context.core_types.i32.into(),
            Type::str => context.core_types.str.into(),
            Type::unit => context.core_types.unit.into(),
            Type::bool => context.core_types.bool.into(),
        }
    }

    pub fn get_dwarf_type<'ctx>(&self, cu: &CodegenUnit<'_, 'ctx>) -> DIType<'ctx> {
        cu.debug_context.get_type(self)
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
                type_ = Type::i32;
            }
            OpCode::Minus => {
                val = builder.build_sub(lhs, rhs, c"");
                type_ = Type::i32;
            }
            OpCode::Asterisk => {
                val = builder.build_mul(lhs, rhs, c"");
                type_ = Type::i32;
            }
            OpCode::Slash => {
                val = builder.build_sdiv(lhs, rhs, c"");
                type_ = Type::i32;
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
