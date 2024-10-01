use std::borrow::Cow;

use wllvm::{attribute::AttrKind, value::Linkage};

use crate::{
    codegen::{
        self,
        codegen_context::{CodegenContext, Crate},
        error,
        namestore::{FunctionInfo, FunctionSignature},
        types::Type,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Visibility},
};

impl<'ctx> CodegenContext<'ctx> {
    pub(super) fn generate_function_declarations(
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
}
