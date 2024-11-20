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
                let Some(ptr) = mut_val.ptr else {
                    return RValue {
                        val: None,
                        type_: mut_val.type_,
                    };
                };

                let Some(llvm_type) = mut_val.type_.llvm_type(&cu.c) else {
                    unreachable!("An uninstantiable type cannot have a value!")
                };

                RValue {
                    val: Some(cu.builder.build_load(llvm_type, ptr, c"")),
                    type_: mut_val.type_,
                }
            }
        }
    }

    pub fn type_(&self) -> &Type {
        match self {
            GenericValue::RValue(RValue { val: _, type_ }) => type_,
            GenericValue::MutValue(MutValue { ptr: _, type_ }) => type_,
        }
    }
}

#[derive(Clone)]
pub struct RValue<'ctx> {
    /// The LLVM value or `None` if the type is `!` or the code that creates it is unreachable
    pub val: Option<wllvm::Value<'ctx>>,
    pub type_: Type,
}

#[derive(Clone)]
pub struct MutValue<'ctx> {
    /// The LLVM value or `None` if the type is `!` or the code that creates it is unreachable
    pub ptr: Option<wllvm::value::PtrValue<'ctx>>,
    pub type_: Type,
}

impl<'ctx> MutValue<'ctx> {
    pub fn alloca(cu: &CodegenUnit<'_, 'ctx>, rvalue: RValue<'ctx>) -> Self {
        let Some(val) = rvalue.val else {
            return Self {
                ptr: None,
                type_: rvalue.type_,
            };
        };

        let Some(llvm_type) = rvalue.type_.llvm_type(&cu.c) else {
            unreachable!("Uninstantiable types cannot have values");
        };

        let ptr = cu.builder.build_alloca(llvm_type, c"");
        cu.builder.build_store(val, ptr);

        Self {
            ptr: Some(ptr),
            type_: rvalue.type_,
        }
    }
}

impl<'ctx> RValue<'ctx> {
    pub fn generate_operation(
        &self,
        cu: &CodegenUnit<'_, 'ctx>,
        lhs_span: Span,
        opcode: OpCode,
        rhs: &S<Self>,
    ) -> Result<Self, Diagnostic> {
        let builder = &cu.builder;
        match self.type_ {
            Type::i(n) => self.generate_operation_int(n, builder, lhs_span, opcode, rhs),
            Type::unit | Type::str | Type::Struct { .. } => {
                Err(error::undefined_operator(opcode, lhs_span, &self.type_))
            }
            Type::bool => {
                if !rhs.type_.is(&Type::bool) {
                    return Err(error::unexpected_type(rhs.1, &Type::bool, &rhs.type_));
                }

                let Some((lhs_val, rhs_val)) = self.val.zip(rhs.val) else {
                    return Ok(RValue {
                        val: None,
                        type_: Type::bool,
                    });
                };

                let Some((ValueEnum::IntValue(lhs), ValueEnum::IntValue(rhs))) =
                    lhs_val.downcast().zip(rhs_val.downcast())
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
                    val: Some(*val),
                })
            }
            Type::never => Err(error::undefined_operator(opcode, lhs_span, &self.type_)),
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
        if !rhs.type_.is(&Type::i(bits)) {
            return Err(error::unexpected_type(rhs.1, &Type::i(bits), &rhs.type_));
        }

        let Some((lhs_val, rhs_val)) = self.val.zip(rhs.val) else {
            return Ok(RValue {
                val: None,
                type_: Type::i(bits),
            });
        };

        let Some((ValueEnum::IntValue(lhs), ValueEnum::IntValue(rhs))) =
            lhs_val.downcast().zip(rhs_val.downcast())
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
            val: Some(*val),
        })
    }
}
