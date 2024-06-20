use crate::{
    codegen::{
        self,
        scope::{FunctionInfo, FunctionSignature, Scope},
        types::{Type, TypedValue},
        CodegenUnit,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::{CodeBlock, Expression, Statement},
};

use inkwell::types::{BasicMetadataTypeEnum, BasicType};

impl<'ctx> CodegenUnit<'ctx> {
    pub fn generate_function<'a: 'ctx>(
        &self,
        fn_name: &str,
        params: &[(&'a str, S<&'a str>)],
        return_type: Type,
        body: S<&'a CodeBlock<'a>>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<(), Diagnostic> {
        let params: Result<Vec<(&'a str, Type)>, _> = params
            .iter()
            .map(|(n, t)| Ok((*n, Type::new(*t)?)))
            .collect();
        let params = params?;

        let llvm_param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|(_, type_)| type_.get_llvm_type(self).into())
            .collect();

        let function = self.module.add_function(
            fn_name,
            return_type
                .get_llvm_type(self)
                .fn_type(&llvm_param_types, false),
            None,
        );

        let main_block = self.context.append_basic_block(function, "");
        self.builder.position_at_end(main_block);

        let mut fn_scope = Scope::new(scope).with_params(&params, function);

        let return_value = self.generate_codeblock(*body, &mut fn_scope)?;

        if return_value.type_ != return_type {
            return Err(codegen::error::incorrect_return_type(
                body,
                &return_type,
                &return_value.type_,
            ));
        }

        self.builder.build_return(Some(&return_value.val)).unwrap();

        scope.create_function(
            fn_name,
            FunctionInfo {
                signature: FunctionSignature {
                    params: params.into_iter().map(|(_, t)| t).collect(),
                    return_type,
                },
                function,
            },
        );

        Ok(())
    }

    /// Generates a codeblock: NOTE: this will NOT create a new scope. The caller should create one for this block
    pub fn generate_codeblock<'a: 'ctx>(
        &self,
        block: &CodeBlock<'a>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let mut statements: &[S<Statement>] = &block.body;
        let implicit_return: Option<S<&Expression>>;

        if block.trailing_semicolon.is_none() {
            if let Some((last_statement, statements_)) = statements.split_last() {
                if let Statement::Expression(expr) = &**last_statement {
                    implicit_return = Some(S(expr, last_statement.1));
                    statements = statements_;
                } else {
                    implicit_return = None;
                };
            } else {
                implicit_return = None;
            };
        } else {
            implicit_return = None;
        }

        for statement in statements {
            self.generate_statement(scope, statement.as_sref())?;
        }

        let return_value: TypedValue = implicit_return
            .map(|r| self.generate_expression(r, scope))
            .transpose()?
            .unwrap_or_else(|| TypedValue {
                type_: Type::unit,
                val: self.core_types.unit.const_zero().into(),
            });

        Ok(return_value)
    }
}
