use inkwell::{
    context::Context,
    targets::{Target, TargetMachine},
};

use crate::codegen::CoreTypes;

pub struct CodegenContext<'ctx> {
    pub(super) target: TargetMachine,
    pub(super) context: &'ctx Context,
    pub(super) core_types: CoreTypes<'ctx>,
}

impl<'ctx> CodegenContext<'ctx> {
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
            target,
            context,
            core_types,
        }
    }
}
