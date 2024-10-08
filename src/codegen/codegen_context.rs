use std::{ffi::CString, path::Path};

use wllvm::{
    target::{self, Target, TargetData, TargetMachine},
    Context, Module as LlvmModule,
};

use crate::{
    cmdline,
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        namestore::{NameStore, StructInfo},
        scope::Scope,
        CoreTypes,
    },
    error_handling::Diagnostic,
    parser::ast::{self},
    util::PushVec,
};

mod functions;
mod structs;

pub struct Crate<'ctx> {
    pub llvm_module: LlvmModule<'ctx>,
    pub name: String,
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
    /// The crate that contains the `main` function
    pub(super) main_crate: Option<String>,
    pub warnings: PushVec<(usize, Diagnostic)>,
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
        let warnings = PushVec::new();

        Self {
            target,
            target_data,
            context,
            core_types,
            name_store,
            files,
            params,
            main_crate: None,
            warnings,
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

        for struct_ in &ast.structs {
            let llvm_type = self.context.create_named_struct(struct_.name);

            self.name_store.add_struct(
                &[crate_name, struct_.name],
                // placeholder values; these will be replaced by Self::generate_struct_bodies
                StructInfo {
                    llvm_type: Some(llvm_type),
                    fields: Vec::new(),
                    packed: false,
                    line_no: 0,
                    file_no: 0,
                },
            );
        }

        Ok(Crate {
            llvm_module: module,
            name: crate_name.into(),
            file_no,
        })
    }

    pub fn add_types_and_functions(
        &mut self,
        ast: &ast::Module,
        source: &str,
        crate_: &Crate<'ctx>,
    ) -> Result<(), Diagnostic> {
        self.generate_struct_bodies(ast, source, crate_)?;
        self.generate_function_declarations(ast, crate_)?;

        Ok(())
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn generate_crate(
        &mut self,
        crate_: &Crate<'ctx>,
        ast: &ast::Module,
        params: &cmdline::Parameters,
        source: &str,
    ) -> Result<(), Diagnostic> {
        let crate_name = &crate_.name;

        let mut generator = CodegenUnit::new(self, crate_, crate_.file_no, source);
        let mut scope = Scope::new_global();

        for function in &ast.functions {
            generator.generate_function(function, &mut scope)?;
        }

        if self
            .main_crate
            .as_deref()
            .is_some_and(|n| n == &crate_.name)
        {
            generator.generate_entrypoint()?;
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

        // if let Err(e) = generator.module.verify() {
        //     std::eprintln!("\n COMPILER BUG: LLVM ERROR:\n");
        //     std::io::stderr().write_all(e.as_bytes()).unwrap();
        //     std::eprintln!();

        //     std::process::abort();
        // }

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
