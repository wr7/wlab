use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        scope::Scope,
        types::{Type, TypedValue},
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Expression, Literal, Path},
};

use inkwell::{
    types::StringRadix,
    values::{BasicMetadataValueEnum, BasicValueEnum},
};

use wutil::Span;

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_expression(
        &self,
        expression: S<&ast::Expression>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        // not in match statement due to rustc bug
        match *expression {
            Expression::Identifier(ident) => match *ident {
                "true" => Ok(TypedValue {
                    val: self.c.core_types.bool.const_int(1, false).into(),
                    type_: Type::bool,
                }),
                "false" => Ok(TypedValue {
                    val: self.c.core_types.bool.const_int(0, false).into(),
                    type_: Type::bool,
                }),
                _ => scope
                    .get_variable(ident)
                    .cloned()
                    .ok_or(codegen::error::undefined_variable(S(ident, expression.1))),
            },
            Expression::Literal(lit) => self.generate_literal(S(lit, expression.1)),
            Expression::BinaryOperator(a_expr, operator, b_expr) => {
                let a = self.generate_expression(a_expr.as_sref(), scope)?;
                let b = self.generate_expression(b_expr.as_sref(), scope)?;

                a.generate_operation(&self.builder, a_expr.1, *operator, &S(b, b_expr.1))
            }
            Expression::CompoundExpression(block) => {
                let mut scope = Scope::new(scope);
                self.generate_codeblock(block, &mut scope)
            }
            Expression::FunctionCall(fn_name, arguments) => {
                self.generate_function_call(expression.1, scope, fn_name, arguments)
            }
            Expression::If {
                condition,
                block,
                else_block,
            } => self.generate_if(scope, condition, block.as_sref(), else_block),
        }
    }

    fn generate_if(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        condition: &S<ast::Expression>,
        block: S<&ast::CodeBlock>,
        else_block: &Option<S<ast::CodeBlock>>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let condition_span = condition.1;
        let condition = self.generate_expression(condition.as_sref(), scope)?;

        if condition.type_ != Type::bool {
            return Err(codegen::error::unexpected_type(
                condition_span,
                &Type::bool,
                &condition.type_,
            ));
        }

        let BasicValueEnum::IntValue(condition) = condition.val else {
            unreachable!()
        };

        let Some(base_bb) = self.current_block.get() else {
            unreachable!()
        };

        let if_bb = self.c.context.insert_basic_block_after(base_bb, "");
        let continuing_bb = self.c.context.insert_basic_block_after(if_bb, "");

        self.position_at_end(if_bb);

        let mut if_scope = Scope::new(scope);
        let if_retval = self.generate_codeblock(*block, &mut if_scope)?;

        self.builder
            .build_unconditional_branch(continuing_bb)
            .unwrap();

        let else_bb;
        let else_retval: Option<TypedValue<'ctx>> = if let Some(else_block) = else_block {
            let else_bb_ = self.c.context.insert_basic_block_after(if_bb, "");
            else_bb = Some(else_bb_);

            self.position_at_end(else_bb_);
            let mut else_scope = Scope::new(scope);

            let else_retval = self.generate_codeblock(else_block, &mut else_scope)?;

            self.builder
                .build_unconditional_branch(continuing_bb)
                .unwrap();

            Some(else_retval)
        } else {
            else_bb = None;
            None
        };

        self.position_at_end(base_bb);
        self.builder
            .build_conditional_branch(condition, if_bb, else_bb.unwrap_or(continuing_bb))
            .unwrap();

        self.position_at_end(continuing_bb);

        let retval = if let Some(else_retval) = else_retval {
            if else_retval.type_ != if_retval.type_ {
                return Err(codegen::error::mismatched_if_else(
                    S(&if_retval.type_, block.1),
                    S(&else_retval.type_, else_block.as_ref().unwrap().1),
                ));
            }

            let phi = self
                .builder
                .build_phi(if_retval.val.get_type(), "")
                .unwrap();

            phi.add_incoming(&[
                (&if_retval.val, if_bb),
                (&else_retval.val, else_bb.unwrap()),
            ]);

            TypedValue {
                type_: if_retval.type_,
                val: phi.as_basic_value(),
            }
        } else {
            TypedValue {
                type_: Type::unit,
                val: self.c.core_types.unit.const_zero().into(),
            }
        };

        Ok(retval)
    }

    fn generate_literal(&self, literal: S<&ast::Literal>) -> Result<TypedValue<'ctx>, Diagnostic> {
        match *literal {
            Literal::Number(num) => self.generate_number_literal(num, literal.1),
            Literal::String(str) => Ok(self.generate_string_literal(str)),
        }
    }

    fn generate_string_literal(&self, lit: &str) -> TypedValue<'ctx> {
        let string = self.c.context.const_string(lit.as_bytes(), false);

        let string_global = self.module.add_global(
            self.c.context.i8_type().array_type(lit.len() as u32),
            None,
            "",
        );

        string_global.set_initializer(&string);
        string_global.set_constant(true);
        string_global.set_linkage(inkwell::module::Linkage::Private);

        let string_ptr = string_global.as_pointer_value();

        let str_len = self.c.core_types.isize.const_int(lit.len() as u64, false);

        TypedValue {
            val: self
                .c
                .core_types
                .str
                .const_named_struct(&[string_ptr.into(), str_len.into()])
                .into(),
            type_: Type::str,
        }
    }

    fn generate_number_literal(
        &self,
        lit: &str,
        span: Span,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        Ok(TypedValue {
            val: self
                .c
                .context
                .i32_type()
                .const_int_from_string(lit, StringRadix::Decimal)
                .ok_or(codegen::error::invalid_number(S(lit, span)))?
                .into(),
            type_: Type::i32,
        })
    }

    fn generate_function_call(
        &self,
        span: Span,
        scope: &mut Scope<'_, 'ctx>,
        fn_name: &S<Path>,
        arguments: &[S<ast::Expression>],
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let function = if let [fn_name] = &***fn_name {
            self.c
                .name_store
                .get_item_in_crate(self.crate_name, *fn_name)?
                .as_function()
                .ok_or_else(|| codegen::error::not_function(*fn_name))?
                .clone()
        } else {
            self.c
                .name_store
                .get_item(fn_name)?
                .as_function()
                .ok_or_else(|| codegen::error::not_function_path(fn_name))?
                .clone()
        };

        let fn_name = &function.name;

        // Add a declaration to this module if it doesn't already exist //
        let mod_function = self.module.get_function(fn_name).unwrap_or_else(|| {
            self.module.add_function(
                fn_name,
                function.function.get_type(),
                Some(inkwell::module::Linkage::External),
            )
        });

        let signature = &function.signature;

        if arguments.len() != signature.params.len() {
            return Err(codegen::error::invalid_param_count(
                span,
                signature.params.len(),
                arguments.len(),
            ));
        }

        let mut metadata_arguments: Vec<BasicMetadataValueEnum> =
            Vec::with_capacity(arguments.len());

        for (i, arg) in arguments.iter().enumerate() {
            let arg = self.generate_expression(arg.as_sref(), scope)?;

            metadata_arguments.push(arg.val.into());

            let expected_type = &signature.params[i];
            if expected_type != &arg.type_ {
                return Err(codegen::error::unexpected_type(
                    arguments[i].1,
                    expected_type,
                    &arg.type_,
                ));
            }
        }

        let ret_val = self
            .builder
            .build_direct_call(mod_function, &metadata_arguments, "")
            .unwrap();

        Ok(TypedValue {
            val: ret_val.try_as_basic_value().left().unwrap(),
            type_: function.signature.return_type,
        })
    }
}
