use std::{borrow::Cow, path::Path};

use inkwell::{
    context::Context,
    module::{Linkage, Module as LlvmModule},
    targets::{Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType as _},
};

use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        intrinsics,
        scope::{FunctionInfo, FunctionSignature, Scope},
        types::Type,
        CoreTypes,
    },
    error_handling::Diagnostic,
    parser::{self, Attribute, Visibility},
};

use super::namestore::NameStore;

pub struct Crate<'ctx> {
    llvm_module: LlvmModule<'ctx>,
    crate_name: String,
}

pub struct CodegenContext<'ctx> {
    pub(super) target: TargetMachine,
    pub(super) context: &'ctx Context,
    pub(super) core_types: CoreTypes<'ctx>,
    pub(super) name_store: NameStore<'ctx>,
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
        let name_store = NameStore::new();

        Self {
            target,
            context,
            core_types,
            name_store,
        }
    }
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn create_crate(&mut self, ast: &parser::Module) -> Result<Crate<'ctx>, Diagnostic> {
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

        let module = self.context.create_module(&crate_name);

        for function in &ast.functions {
            let params: Result<Vec<(&str, Type)>, _> = function
                .params
                .iter()
                .map(|(n, t)| Ok((*n, Type::new(*t)?)))
                .collect();
            let params = params?;

            let llvm_param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
                .iter()
                .map(|(_, type_)| type_.get_llvm_type(self).into())
                .collect();

            let return_type = function.return_type.map_or(Ok(Type::unit), Type::new)?;

            let mut no_mangle = false;

            for attr in &function.attributes {
                match **attr {
                    Attribute::DeclareCrate(_) => {
                        return Err(codegen::error::non_function_attribute(attr))
                    }
                    Attribute::NoMangle => no_mangle = true,
                }
            }

            let private = function.visibility == Visibility::Private && !no_mangle;

            let ll_function = module.add_function(
                &if no_mangle {
                    Cow::from(function.name)
                } else {
                    Cow::from(format!("{crate_name}::{}", function.name))
                },
                return_type
                    .get_llvm_type(self)
                    .fn_type(&llvm_param_types, false),
                private.then_some(Linkage::Internal),
            );

            let crate_name: &str = &crate_name;

            if !self.name_store.add_function(
                &[crate_name, function.name],
                FunctionInfo {
                    signature: FunctionSignature {
                        params: params.into_iter().map(|(_, t)| t).collect(),
                        return_type,
                    },
                    function: ll_function,
                },
            ) {
                return Err(codegen::error::function_already_defined(function));
            }
        }

        Ok(Crate {
            llvm_module: module,
            crate_name,
        })
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn generate_crate(
        &mut self,
        crate_: &Crate<'ctx>,
        ast: &parser::Module,
    ) -> Result<(), Diagnostic> {
        let crate_name = &crate_.crate_name;

        let mut generator = CodegenUnit::new(self, &crate_.llvm_module, crate_name);
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
