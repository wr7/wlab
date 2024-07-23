use std::cell::Cell;

use inkwell::{basic_block::BasicBlock, builder::Builder, module::Module as LlvmModule};

use crate::{
    codegen::{codegen_context::CodegenContext, codegen_unit::debug::DebugContext, scope::Scope},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Statement},
};

pub(super) mod debug;
mod expression;
mod function;

pub struct CodegenUnit<'m, 'ctx> {
    pub(super) c: &'m mut CodegenContext<'ctx>,
    pub(super) module: &'m LlvmModule<'ctx>,
    pub(super) debug_context: DebugContext<'ctx>,
    pub(super) builder: Builder<'ctx>,
    pub(super) current_block: Cell<Option<BasicBlock<'ctx>>>,
    pub(super) crate_name: &'m str,
    pub(super) source: &'m str,
}

impl<'m, 'ctx> CodegenUnit<'m, 'ctx> {
    pub fn new(
        c: &'m mut CodegenContext<'ctx>,
        module: &'m LlvmModule<'ctx>,
        crate_name: &'m str,
        file_path: &str,
        source: &'m str,
    ) -> Self {
        let context = c.context;
        let debug_context = DebugContext::new(c, module, file_path);

        Self {
            c,
            module,
            debug_context,
            builder: context.create_builder(),
            current_block: None.into(),
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

    pub(super) fn position_at_end(&self, basic_block: BasicBlock<'ctx>) {
        self.builder.position_at_end(basic_block);
        self.current_block.replace(Some(basic_block));
    }
}
