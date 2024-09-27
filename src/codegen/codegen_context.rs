use std::{borrow::Cow, ffi::CString, path::Path};

use wllvm::{
    attribute::AttrKind,
    target::{self, Target, TargetData, TargetMachine},
    value::Linkage,
    Context, Module as LlvmModule,
};

use crate::{
    cmdline,
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        error,
        namestore::{FunctionInfo, FunctionSignature, NameStore},
        scope::Scope,
        types::Type,
        CoreTypes,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Visibility},
    util::{self, PushVec},
};

use super::namestore::{FieldInfo, StructInfo};

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
        source: &str,
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
            let mut packed = false;

            for attr in &struct_.attributes {
                match &**attr {
                    ast::Attribute::Packed => packed = true,
                    _ => return Err(codegen::error::non_struct_attribute(attr)),
                }
            }

            let line_no = util::line_and_col(source, struct_.1.start).0 as u32;

            let mut fields = Vec::new();
            let mut field_names: Vec<S<&str>> = Vec::new();

            for field in &struct_.fields {
                match field_names.binary_search_by(|f| f.cmp(field.name)) {
                    Ok(idx) => {
                        let field1 = field_names[idx];
                        return Err(codegen::error::duplicate_field(field1, field.1));
                    }
                    Err(idx) => field_names.insert(idx, S(field.name, field.1)),
                }

                let line_no = util::line_and_col(source, field.1.start).0 as u32;
                let ty = Type::new(self, crate_name, &field.type_)?;

                fields.push(FieldInfo {
                    name: field.name.to_owned(),
                    ty,
                    line_no,
                });
            }

            if !self.name_store.add_struct(
                &[crate_name, struct_.name],
                StructInfo {
                    fields,
                    packed,
                    line_no,
                    file_no,
                },
            ) {
                return Err(codegen::error::item_already_defined(S(
                    struct_.name,
                    struct_.1,
                )));
            }
        }

        Ok(Crate {
            llvm_module: module,
            name: crate_name.into(),
            file_no,
        })
    }

    pub fn add_functions(
        &mut self,
        ast: &ast::Module,
        crate_: &Crate<'ctx>,
    ) -> Result<(), Diagnostic> {
        let crate_name = &*crate_.name;
        let module = &crate_.llvm_module;

        for function in &ast.functions {
            let params: Result<Vec<(S<&str>, Type)>, _> = function
                .params
                .iter()
                .map(|(n, t)| Ok((*n, Type::new(self, &crate_.name, t)?)))
                .collect();
            let params = params?;

            let llvm_param_types: Vec<wllvm::Type<'ctx>> = params
                .iter()
                .map(|(_, type_)| type_.llvm_type(self).unwrap_or(*self.core_types.unit))
                .collect();

            let return_type = function
                .return_type
                .as_ref()
                .map_or(Ok(Type::unit), |t| Type::new(self, &crate_.name, t))?;

            let mut no_mangle = false;

            for attr in &function.attributes {
                match **attr {
                    ast::Attribute::NoMangle => no_mangle = true,
                    ast::Attribute::Intrinsic(_) => {}
                    _ => return Err(codegen::error::non_function_attribute(attr)),
                }
            }

            let private = function.visibility == Visibility::Private && !no_mangle;

            let fn_name = if no_mangle {
                Cow::from(function.name)
            } else {
                Cow::from(format!("_WL@{crate_name}::{}", function.name))
            };

            let llvm_return_type = return_type.llvm_type(self);

            let ll_function = module.add_function(
                c"",
                self.context.fn_type(
                    llvm_return_type.unwrap_or(*self.core_types.unit),
                    &llvm_param_types,
                    false,
                ),
            );

            if llvm_return_type.is_none() {
                ll_function.add_attribute(self.context.attribute(AttrKind::NoReturn()));
            }

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
                },
            ) {
                return Err(codegen::error::item_already_defined(S(
                    function.name,
                    function.1,
                )));
            }

            if function.name == "main" {
                if let Some(other_crate) = &self.main_crate {
                    return Err(error::duplicate_main(&other_crate, crate_name, function.1));
                }

                self.main_crate = Some(crate_name.to_owned());

                if !function.params.is_empty() {
                    return Err(error::main_arguments(function.params.1));
                }

                if let Some(return_type) = &function.return_type {
                    if return_type.get(0).is_some_and(|t| **t != "()") {
                        return Err(error::main_return_type(return_type.1));
                    }
                }
            }
        }

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
