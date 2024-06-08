use crate::{
    codegen::{
        error::CodegenError,
        scope::{FunctionInfo, Scope},
        types::Type,
        CodegenUnit,
    },
    error_handling::Spanned as S,
    parser::Statement,
};

use inkwell::types::BasicMetadataTypeEnum;

impl<'ctx> CodegenUnit<'ctx> {
    pub fn generate_function<'a: 'ctx>(
        &self,
        fn_name: &str,
        params: &[(&'a str, &'a str)],
        body: &[S<Statement<'a>>],
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<(), CodegenError<'a>> {
        let params: Result<Vec<(&'a str, Type)>, _> = params
            .iter()
            .map(|(n, t)| Ok((*n, Type::new(t)?)))
            .collect();
        let params = params?;

        let llvm_param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|(_, type_)| type_.get_llvm_type(&self).into())
            .collect();

        let function = self.module.add_function(
            &fn_name,
            self.core_types.unit.fn_type(&llvm_param_types, false),
            None,
        );

        let main_block = self.context.append_basic_block(function, "");
        self.builder.position_at_end(main_block);

        let zero = self.core_types.unit.const_zero();

        let mut fn_scope = Scope::new(&scope).with_params(&params, &function);

        for statement in body {
            self.generate_statement(&mut fn_scope, statement)?;
        }

        self.builder.build_return(Some(&zero)).unwrap();

        // TODO: function already defined error?

        scope.create_function(
            fn_name,
            FunctionInfo {
                params: params.into_iter().map(|(_, t)| t).collect(),
                function,
            },
        );

        Ok(())
    }
}
