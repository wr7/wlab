use std::path::Path;

use inkwell::{
    context::Context,
    targets::{Target, TargetMachine},
};

use crate::{
    codegen::{self, codegen_unit::CodegenUnit, intrinsics, scope::Scope, CoreTypes},
    error_handling::Diagnostic,
    parser::Module,
};

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

impl<'ctx> CodegenContext<'ctx> {
    pub fn generate_code<'a, 'b: 'ctx>(
        &'a mut self,
        ast: &'b Module<'b>,
    ) -> Result<(), Diagnostic> {
        let mut crate_name = None;
        for attr in &ast.attributes {
            match **attr {
                crate::parser::Attribute::DeclareCrate(ref name) => crate_name = Some(name.clone()),
                _ => return Err(codegen::error::non_module_attribute(attr)),
            }
        }

        let Some(crate_name) = crate_name else {
            return Err(codegen::error::missing_crate_name());
        };

        let mut generator = CodegenUnit::new(self);
        let mut scope = Scope::new_global();

        intrinsics::add_intrinsics(&generator, &mut scope);

        for function in &ast.functions {
            generator.generate_function(function, &mut scope)?;
        }

        let llvm_ir = generator.module.to_string();

        std::fs::write(format!("./compiler_output/{crate_name}.ll"), llvm_ir).unwrap();

        generator
            .c
            .target
            .write_to_file(
                &generator.module,
                inkwell::targets::FileType::Object,
                Path::new(&format!("./compiler_output/{crate_name}.o")),
            )
            .unwrap();

        generator
            .c
            .target
            .write_to_file(
                &generator.module,
                inkwell::targets::FileType::Assembly,
                Path::new(&format!("./compiler_output/{crate_name}.asm")),
            )
            .unwrap();

        Ok(())
    }
}
