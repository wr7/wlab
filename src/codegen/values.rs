use wllvm::{builder::IntPredicate, value::ValueEnum, Builder};
use wutil::Span;

use crate::{
    codegen::{error, types::Type},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::OpCode,
};

use super::codegen_unit::CodegenUnit;

#[derive(Clone)]
pub enum GenericValue<'ctx> {
    RValue(RValue<'ctx>),
    MutValue(MutValue<'ctx>),
}

impl<'ctx> GenericValue<'ctx> {
    pub fn into_rvalue(self, cu: &CodegenUnit<'_, 'ctx>) -> RValue<'ctx> {
        match self {
            GenericValue::RValue(rval) => rval,
            GenericValue::MutValue(mut_val) => {
                let val = cu
                    .builder
                    .build_load(mut_val.type_.llvm_type(cu.c), mut_val.ptr, c"");

                RValue {
                    val,
                    type_: mut_val.type_,
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RValue<'ctx> {
    pub val: wllvm::Value<'ctx>,
    pub type_: Type,
}

#[derive(Clone)]
pub struct MutValue<'ctx> {
    pub ptr: wllvm::value::PtrValue<'ctx>,
    pub type_: Type,
}

impl<'ctx> RValue<'ctx> {
    pub fn generate_operation(
        &self,
        builder: &Builder<'ctx>,
        lhs_span: Span,
        opcode: OpCode,
        rhs: &S<Self>,
    ) -> Result<Self, Diagnostic> {
        match self.type_ {
            Type::i(n) => self.generate_operation_int(n, builder, lhs_span, opcode, rhs),
            Type::unit | Type::str | Type::Struct { .. } => {
                Err(error::undefined_operator(opcode, lhs_span, &self.type_))
            }
            Type::bool => {
                if rhs.type_ != Type::bool {
                    return Err(error::unexpected_type(rhs.1, &Type::bool, &rhs.type_));
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
                    _ => return Err(error::undefined_operator(opcode, lhs_span, &self.type_)),
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
        rhs: &S<RValue<'ctx>>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        if rhs.type_ != Type::i(bits) {
            return Err(error::unexpected_type(rhs.1, &Type::i(bits), &rhs.type_));
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
                return Err(error::undefined_operator(opcode, lhs_span, &self.type_))
            }
        };

        Ok(Self {
            type_,
            val: val.into(),
        })
    }
}
