use wllvm::{Builder, Module as LlvmModule};

use crate::{
    codegen::{codegen_context::CodegenContext, codegen_unit::debug::DebugContext, scope::Scope},
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
        let crate_name = &crate_.crate_name;
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
            Statement::Let(varname, val) => {
                let val = self.generate_expression(val.as_sref(), scope)?;
                scope.create_variable(varname, val);
            }
            Statement::Assign(_, _) => todo!(),
            Statement::Function(_) => todo!(),
        }
        Ok(())
    }
}
