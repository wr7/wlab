use wllvm::{Builder, Module as LlvmModule};

use crate::{
    codegen::{
        codegen_context::{CodegenContext, Crate},
        codegen_unit::debug::DebugContext,
        error,
        scope::Scope,
        values::{GenericValue, MutValue, RValue},
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Statement},
};

use super::types::Type;

pub(super) mod debug;
mod expression;
mod function;
mod main;

pub struct CodegenUnit<'m, 'ctx> {
    pub(super) c: &'m CodegenContext<'ctx>,
    pub(super) module: &'m LlvmModule<'ctx>,
    pub(super) debug_context: DebugContext<'ctx>,
    pub(super) builder: Builder<'ctx>,
    pub(super) crate_name: &'m str,
    pub(super) source: &'m str,
    pub(super) file_no: usize,
}

impl<'m, 'ctx> CodegenUnit<'m, 'ctx> {
    pub fn new(
        c: &'m CodegenContext<'ctx>,
        crate_: &'m Crate<'ctx>,
        file_no: usize,
        source: &'m str,
    ) -> Self {
        let context = c.context;
        let module = &crate_.llvm_module;
        let crate_name = &crate_.name;
        let debug_context = DebugContext::new(c, module, file_no);

        Self {
            c,
            module,
            debug_context,
            builder: context.create_builder(),
            crate_name,
            source,
            file_no,
        }
    }

    pub fn generate_statement(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        statement: S<&ast::Statement>,
    ) -> Result<Option<RValue>, Diagnostic> {
        match *statement {
            Statement::Expression(expr) => self
                .generate_rvalue(S(expr, statement.1), scope)
                .map(|v| Some(v)),
            Statement::Let {
                name,
                value,
                mutable,
            } => {
                let orig_val = self.generate_rvalue(value.as_sref(), scope)?;

                let unreachable = orig_val.val.is_none();

                let val = if *mutable {
                    GenericValue::MutValue(MutValue::alloca(self, orig_val))
                } else {
                    GenericValue::RValue(orig_val)
                };

                scope.create_variable(*name, val);
                Ok(unreachable.then_some(RValue {
                    val: None,
                    type_: Type::never,
                }))
            }
            Statement::Struct(_) => todo!(),
            Statement::Assign { lhs, rhs } => {
                let lhs_val = self.generate_mutvalue(lhs.as_sref(), scope)?;
                let rhs_val = self.generate_rvalue(rhs.as_sref(), scope)?;

                let unreachable = lhs_val.ptr.is_none() || rhs_val.val.is_none();

                if !rhs_val.type_.is(&lhs_val.type_) {
                    return Err(error::unexpected_type(
                        rhs.1,
                        &lhs_val.type_,
                        &rhs_val.type_,
                    ));
                }

                let Some((lhs_ptr, rhs_val)) = lhs_val.ptr.zip(rhs_val.val) else {
                    return Ok(None);
                };

                self.builder.build_store(rhs_val, lhs_ptr);
                Ok(unreachable.then_some(RValue {
                    val: None,
                    type_: Type::never,
                }))
            }
            Statement::Function(_) => todo!(),
        }
    }
}
