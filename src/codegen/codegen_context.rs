use std::{borrow::Cow, ffi::CString, path::Path};

use wllvm::{
    target::{self, Target, TargetData, TargetMachine},
    value::Linkage,
    Context, Module as LlvmModule,
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
    pub llvm_module: LlvmModule<'ctx>,
    pub crate_name: String,
    pub file_no: usize,
}

pub struct CodegenContext<'ctx> {
    pub(super) target: TargetMachine,
    pub(super) target_data: TargetData,
    pub(super) context: &'ctx Context,
    pub(super) core_types: CoreTypes<'ctx>,
    pub(super) name_store: NameStore<'ctx>,
    pub(super) files: Vec<String>,
    pub(super) params: &'ctx cmdline::Parameters,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, params: &'ctx cmdline::Parameters) -> Self {
        if !Target::initialize_native(true, true, true) {
            panic!("native target not supported") // todo_panic
        }

        let target_triple = target::host_target_triple();

        let target = Target::from_triple(&target_triple).unwrap();
        let target = target.create_target_machine(
            &target_triple,
            &target::host_cpu(),
            &target::host_cpu_features(),
            params.opt_level,
            Default::default(),
            Default::default(),
        );

        let target_data = target.create_target_data();

        let core_types = CoreTypes::new(context, &target_data);
        let name_store = NameStore::new();
        let files = Vec::new();

        Self {
            target,
            target_data,
            context,
            core_types,
            name_store,
            files,
            params,
        }
    }
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn create_crate(
        &mut self,
        ast: &ast::Module,
        file_name: String,
    ) -> Result<Crate<'ctx>, Diagnostic> {
        self.files.push(file_name);
        let file_no = self.files.len() - 1;

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

        let module = self
            .context
            .create_module(&CString::new(crate_name).unwrap());

        for function in &ast.functions {
            let params: Result<Vec<(&str, Type)>, _> = function
                .params
                .iter()
                .map(|(n, t)| Ok((*n, Type::new(*t)?)))
                .collect();
            let params = params?;

            let llvm_param_types: Vec<wllvm::Type<'ctx>> = params
                .iter()
                .map(|(_, type_)| type_.get_llvm_type(self))
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
                c"",
                self.context
                    .fn_type(return_type.get_llvm_type(self), &llvm_param_types, false),
            );

            ll_function.set_name(&*fn_name);
            ll_function.set_linkage(if private {
                Linkage::Internal
            } else {
                Linkage::External
            });

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
            file_no,
        })
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn generate_crate(
        &mut self,
        crate_: &Crate<'ctx>,
        ast: &ast::Module,
        params: &cmdline::Parameters,
        source: &str,
    ) -> Result<(), Diagnostic> {
        let crate_name = &crate_.crate_name;

        let mut generator = CodegenUnit::new(self, crate_, crate_.file_no, source);
        let mut scope = Scope::new_global();

        for function in &ast.functions {
            generator.generate_function(function, &mut scope)?;
        }

        generator.debug_context.builder.finalize();

        if params.generate_ir {
            let llvm_ir = generator.module.print_to_string();
            std::fs::write(
                format!("{}/{crate_name}.ll", &*params.out_dir),
                llvm_ir.as_bytes(),
            )
            .unwrap();
        }

        if params.generate_asm {
            let asm = generator
                .module
                .compile_to_buffer(&generator.c.target, true)
                .unwrap();
            std::fs::write(
                Path::new(&format!("{}/{crate_name}.asm", &*params.out_dir)),
                &asm,
            )
            .unwrap();
        }

        if params.generate_object {
            let object = generator
                .module
                .compile_to_buffer(&generator.c.target, false)
                .unwrap();
            std::fs::write(
                Path::new(&format!("{}/{crate_name}.o", &*params.out_dir)),
                &object,
            )
            .unwrap();
        }

        Ok(())
    }
}
