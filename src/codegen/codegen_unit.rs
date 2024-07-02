use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{Target, TargetMachine},
};

use crate::{
    codegen::{scope::Scope, CoreTypes},
    error_handling::{Diagnostic, Spanned as S},
    parser::Statement,
};

mod expression;
mod function;

pub struct CodegenUnit<'ctx> {
    pub(super) target: TargetMachine,
    pub(super) context: &'ctx Context,
    pub(super) builder: Builder<'ctx>,
    pub(super) core_types: CoreTypes<'ctx>,
    pub(super) module: Module<'ctx>,
    pub(super) current_block: Option<BasicBlock<'ctx>>,
}

impl<'ctx> CodegenUnit<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Target::initialize_native(&Default::default()).unwrap();
        let target = Target::get_first().unwrap();
        let target = target
            .create_target_machine(
                &TargetMachine::get_default_triple(),
                TargetMachine::get_host_cpu_name().to_str().unwrap(),
                TargetMachine::get_host_cpu_features().to_str().unwrap(),
                inkwell::OptimizationLevel::Default,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .unwrap();

        let core_types = CoreTypes::new(context, &target);

        Self {
            context,
            target,
            module: context.create_module("my_module"),
            builder: context.create_builder(),
            core_types,
            current_block: None,
        }
    }

    pub fn generate_statement<'a: 'ctx>(
        &mut self,
        scope: &mut Scope<'_, 'ctx>,
        statement: S<&Statement<'a>>,
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
