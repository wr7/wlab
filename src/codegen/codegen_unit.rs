use std::mem;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    targets::{Target, TargetMachine},
};

use crate::{
    codegen::{error::CodegenError, scope::Scope, CoreTypes},
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
        }
    }

    pub fn generate_statement<'a: 'ctx>(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        statement: &Statement<'a>,
    ) -> Result<(), CodegenError<'a>> {
        match statement {
            Statement::Expression(expr) => mem::drop(self.generate_expression(expr, scope)?),
            Statement::Let(varname, val) => {
                let val = self.generate_expression(val, scope)?;
                scope.create_variable(varname, val);
            }
            Statement::Assign(_, _) => todo!(),
            Statement::Function(_, _, _) => todo!(),
        }
        Ok(())
    }
}