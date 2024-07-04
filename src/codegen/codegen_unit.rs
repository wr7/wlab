use inkwell::{basic_block::BasicBlock, builder::Builder, module::Module};

use crate::{
    codegen::{codegen_context::CodegenContext, scope::Scope},
    error_handling::{Diagnostic, Spanned as S},
    parser::Statement,
};

mod expression;
mod function;

pub struct CodegenUnit<'m, 'ctx> {
    pub(super) c: &'m mut CodegenContext<'ctx>,
    pub(super) builder: Builder<'ctx>,
    pub(super) module: Module<'ctx>,
    pub(super) current_block: Option<BasicBlock<'ctx>>,
}

impl<'m, 'ctx> CodegenUnit<'m, 'ctx> {
    pub fn new(c: &'m mut CodegenContext<'ctx>) -> Self {
        let context = c.context;

        Self {
            c,
            module: context.create_module("my_module"),
            builder: context.create_builder(),
            current_block: None,
        }
    }

    pub fn generate_statement(
        &mut self,
        scope: &mut Scope<'_, 'ctx>,
        statement: S<&Statement>,
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

    pub(super) fn position_at_end(&mut self, basic_block: BasicBlock<'ctx>) {
        self.builder.position_at_end(basic_block);
        self.current_block = Some(basic_block);
    }
}
