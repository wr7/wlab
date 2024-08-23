use wllvm::{Builder, Module as LlvmModule};

use crate::{
    codegen::{
        codegen_context::{CodegenContext, Crate},
        codegen_unit::debug::DebugContext,
        error,
        scope::Scope,
        values::{GenericValue, MutValue},
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Statement},
};

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
        }
    }

    pub fn generate_statement(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        statement: S<&ast::Statement>,
    ) -> Result<(), Diagnostic> {
        match *statement {
            Statement::Expression(expr) => {
                self.generate_rvalue(S(expr, statement.1), scope)?;
            }
            Statement::Let {
                name,
                value,
                mutable,
            } => {
                let val = self.generate_rvalue(value.as_sref(), scope)?;

                let val = if *mutable {
                    let ptr = self.builder.build_alloca(val.val.type_(), c"");
                    self.builder.build_store(val.val, ptr);

                    GenericValue::MutValue(MutValue {
                        ptr,
                        type_: val.type_,
                    })
                } else {
                    GenericValue::RValue(val)
                };

                scope.create_variable(*name, val);
            }
            Statement::Struct(_) => todo!(),
            Statement::Assign { lhs, rhs } => {
                let lhs_val = self.generate_mutvalue(lhs.as_sref(), scope)?;
                let rhs_val = self.generate_rvalue(rhs.as_sref(), scope)?;

                if lhs_val.type_ != rhs_val.type_ {
                    return Err(error::unexpected_type(
                        rhs.1,
                        &lhs_val.type_,
                        &rhs_val.type_,
                    ));
                }

                self.builder.build_store(rhs_val.val, lhs_val.ptr);
            }
            Statement::Function(_) => todo!(),
        }
        Ok(())
    }
}
