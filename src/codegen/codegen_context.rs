use std::{borrow::Cow, path::Path};

use inkwell::{
    context::Context,
    module::{Linkage, Module as LlvmModule},
    targets::{Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType as _},
};

use crate::{
    cmdline,
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        namestore::{FunctionInfo, FunctionSignature, NameStore},
        scope::Scope,
        types::Type,
        CoreTypes,
    },
    error_handling::Diagnostic,
    parser::ast::{self, Visibility},
};

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
    pub fn create_crate(&mut self, ast: &ast::Module) -> Result<Crate<'ctx>, Diagnostic> {
        let mut crate_name = None;
        for attr in &ast.attributes {
            match **attr {
                crate::parser::ast::Attribute::DeclareCrate(name) => crate_name = Some(name),
                _ => return Err(codegen::error::non_module_attribute(attr)),
            }
        }

        let Some(crate_name) = crate_name else {
            return Err(codegen::error::missing_crate_name());
        };

        let module = self.context.create_module(crate_name);

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
                    ast::Attribute::DeclareCrate(_) => {
                        return Err(codegen::error::non_function_attribute(attr))
                    }
                    ast::Attribute::NoMangle => no_mangle = true,
                    ast::Attribute::Intrinsic(_) => {}
                }
            }

            let private = function.visibility == Visibility::Private && !no_mangle;

            let fn_name = if no_mangle {
                Cow::from(function.name)
            } else {
                Cow::from(format!("_WL@{crate_name}::{}", function.name))
            };

            let ll_function = module.add_function(
                &fn_name,
                return_type
                    .get_llvm_type(self)
                    .fn_type(&llvm_param_types, false),
                private.then_some(Linkage::Internal),
            );

            if !self.name_store.add_function(
                &[crate_name, function.name],
                FunctionInfo {
                    signature: FunctionSignature {
                        params: params.into_iter().map(|(_, t)| t).collect(),
                        return_type,
                    },
                    function: ll_function,
                    visibility: function.visibility,
                    name: fn_name.into(),
                },
            ) {
                return Err(codegen::error::function_already_defined(function));
            }
        }

        Ok(Crate {
            llvm_module: module,
            crate_name: crate_name.into(),
        })
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn generate_crate(
        &mut self,
        crate_: &Crate<'ctx>,
        ast: &ast::Module,
        params: &cmdline::Parameters,
    ) -> Result<(), Diagnostic> {
        let crate_name = &crate_.crate_name;

        let mut generator = CodegenUnit::new(self, &crate_.llvm_module, crate_name);
        let mut scope = Scope::new_global();

        for function in &ast.functions {
            generator.generate_function(function, &mut scope)?;
        }

        if params.generate_ir {
            let llvm_ir = generator.module.to_string();
            std::fs::write(format!("{}/{crate_name}.ll", &*params.out_dir), llvm_ir).unwrap();
        }

        if params.generate_asm {
            generator
                .c
                .target
                .write_to_file(
                    generator.module,
                    inkwell::targets::FileType::Assembly,
                    Path::new(&format!("{}/{crate_name}.asm", &*params.out_dir)),
                )
                .unwrap();
        }

        if params.generate_object {
            generator
                .c
                .target
                .write_to_file(
                    generator.module,
                    inkwell::targets::FileType::Object,
                    Path::new(&format!("{}/{crate_name}.o", &*params.out_dir)),
                )
                .unwrap();
        }

        Ok(())
    }
}
