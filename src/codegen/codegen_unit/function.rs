use crate::{
    codegen::{
        self,
        scope::{FunctionInfo, FunctionSignature, Scope},
        types::{Type, TypedValue},
        CodegenUnit,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::{Attribute, CodeBlock, Expression, Function, Statement, Visibility},
};

use inkwell::{
    module::Linkage,
    types::{BasicMetadataTypeEnum, BasicType},
};

impl<'m, 'ctx> CodegenUnit<'m, 'ctx> {
    pub fn generate_function<'a: 'ctx>(
        &mut self,
        function: &Function<'a>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<(), Diagnostic> {
        let params: Result<Vec<(&'a str, Type)>, _> = function
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

        let ll_function = self.module.add_function(
            if no_mangle { function.name } else { "" },
            return_type
                .get_llvm_type(self)
                .fn_type(&llvm_param_types, false),
            private.then_some(Linkage::Internal),
        );

        let main_block = self.c.context.append_basic_block(ll_function, "");
        self.position_at_end(main_block);

        let mut fn_scope = Scope::new(scope).with_params(&params, ll_function);

        let return_value = self.generate_codeblock(&function.body, &mut fn_scope)?;

        if return_value.type_ != return_type {
            return Err(codegen::error::incorrect_return_type(
                function.body.as_sref(),
                &return_type,
                &return_value.type_,
            ));
        }

        self.builder.build_return(Some(&return_value.val)).unwrap();

        scope.create_function(
            function.name,
            FunctionInfo {
                signature: FunctionSignature {
                    params: params.into_iter().map(|(_, t)| t).collect(),
                    return_type,
                },
                function: ll_function,
            },
        );

        Ok(())
    }

    /// Generates a codeblock: NOTE: this will NOT create a new scope. The caller should create one for this block
    pub fn generate_codeblock<'a: 'ctx>(
        &mut self,
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
                val: self.c.core_types.unit.const_zero().into(),
            });

        Ok(return_value)
    }
}
