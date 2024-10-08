use std::cell::OnceCell;
use wllvm::{value::PhiValue, BasicBlock};
use wutil::Span;

use crate::{
    codegen::{codegen_unit::CodegenUnit, error, types::Type, values::RValue},
    error_handling::{Diagnostic, Spanned as S},
};

struct BreakPhiValue<'ctx> {
    /// The phi node (or `None` if the type is uninstantiable)
    value: Option<PhiValue<'ctx>>,
    type_: Type,
    defining_span: Span,
}

/// Contains information required for `break` statements
pub struct BreakContext<'ctx> {
    jump_to: BasicBlock<'ctx>,
    phi: OnceCell<BreakPhiValue<'ctx>>,
}

impl<'ctx> BreakContext<'ctx> {
    /// NOTE: this type assumes that the `jump_to`'s only incoming branches are `break`s registered
    /// to this `BreakContext`.
    pub fn new(jump_to: BasicBlock<'ctx>) -> Self {
        Self {
            jump_to,
            phi: OnceCell::new(),
        }
    }

    pub fn build_break(
        &self,
        cu: &CodegenUnit<'_, 'ctx>,
        rvalue: Option<RValue<'ctx>>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let rvalue = rvalue.unwrap_or_else(|| RValue {
            val: Some(*cu.c.context.const_struct(&[], false)),
            type_: Type::unit,
        });

        if rvalue.type_ == Type::never {
            return Ok(());
        }

        let break_phi = self.phi.get_or_init(|| {
            let curr = cu.builder.current_block().unwrap();
            cu.builder.position_at_end(self.jump_to);

            let v = BreakPhiValue {
                value: rvalue
                    .type_
                    .llvm_type(&cu.c)
                    .map(|ty| cu.builder.build_phi(ty, c"")),
                type_: rvalue.type_.clone(),
                defining_span: span,
            };
            cu.builder.position_at_end(curr);
            v
        });

        if !rvalue.type_.is(&break_phi.type_) {
            return Err(error::unexpected_break_type(
                S(&break_phi.type_, break_phi.defining_span),
                S(&rvalue.type_, span),
            ));
        }

        if let Some((phi, val)) = break_phi.value.zip(rvalue.val) {
            cu.builder.build_br(self.jump_to);
            phi.add_incoming(&[val], &[cu.builder.current_block().unwrap()]);
        }

        Ok(())
    }

    pub fn into_rvalue(self) -> RValue<'ctx> {
        let Some(phi) = self.phi.into_inner() else {
            return RValue {
                val: None,
                type_: Type::never,
            };
        };

        let val = if phi.value.is_some_and(|v| v.num_incoming() > 0) {
            phi.value.as_deref().copied()
        } else {
            None
        };

        RValue {
            val,
            type_: phi.type_,
        }
    }
}
