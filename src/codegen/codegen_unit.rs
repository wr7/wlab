use wllvm::{value::PtrValue, Builder, Module as LlvmModule};

use crate::{
    codegen::{
        codegen_context::CodegenContext, codegen_unit::debug::DebugContext, error, scope::Scope,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Statement},
};

use super::codegen_context::Crate;

pub(super) mod debug;
mod expression;
mod function;

pub struct CodegenUnit<'m, 'ctx> {
    pub(super) c: &'m mut CodegenContext<'ctx>,
    pub(super) module: &'m LlvmModule<'ctx>,
    pub(super) debug_context: DebugContext<'ctx>,
    pub(super) builder: Builder<'ctx>,
    pub(super) crate_name: &'m str,
    pub(super) source: &'m str,
}

impl<'m, 'ctx> CodegenUnit<'m, 'ctx> {
    pub fn new(
        c: &'m mut CodegenContext<'ctx>,
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
                self.generate_expression(S(expr, statement.1), scope)?;
            }
            Statement::Let {
                name,
                value,
                mutable,
            } => {
                let mut val = self.generate_expression(value.as_sref(), scope)?;

                if *mutable {
                    let ptr = self.builder.build_alloca(val.val.type_(), c"");
                    self.builder.build_store(val.val, ptr);

                    val.val = *ptr;
                }

                scope.create_variable(*name, val, *mutable);
            }
            Statement::Struct(_) => todo!(),
            Statement::Assign(var_name, val) => {
                let val_span = val.1;
                let val = self.generate_expression(val.as_sref(), scope)?;
                let variable = scope
                    .get_variable(**var_name)
                    .ok_or_else(|| error::undefined_variable(*var_name))?;

                if !variable.mutable {
                    return Err(error::mutate_immutable_variable(
                        S(**var_name, variable.name_span),
                        statement.1,
                    ));
                }

                if variable.value.type_ != val.type_ {
                    return Err(error::unexpected_type(
                        val_span,
                        &variable.value.type_,
                        &val.type_,
                    ));
                }

                let Ok(var_ptr) = PtrValue::try_from(variable.value.val) else {
                    unreachable!()
                };

                self.builder.build_store(val.val, var_ptr);
            }
            Statement::Function(_) => todo!(),
        }
        Ok(())
    }
}
