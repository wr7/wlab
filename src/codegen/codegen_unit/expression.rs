use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        scope::Scope,
        types::{Type, TypedValue},
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::{Expression, Literal},
};

use inkwell::{types::StringRadix, values::BasicMetadataValueEnum};
use wutil::Span;

impl<'ctx> CodegenUnit<'ctx> {
    pub fn generate_expression<'a: 'ctx>(
        &self,
        expression: S<&Expression<'a>>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        match &*expression {
            Expression::Identifier(ident) => match *ident {
                "true" => Ok(TypedValue {
                    val: self.core_types.bool.const_int(1, false).into(),
                    type_: Type::bool,
                }),
                "false" => Ok(TypedValue {
                    val: self.core_types.bool.const_int(0, false).into(),
                    type_: Type::bool,
                }),
                _ => scope
                    .get_variable(ident)
                    .cloned()
                    .ok_or(codegen::error::undefined_variable(S(ident, expression.1))),
            },
            Expression::Literal(Literal::Number(lit)) => {
                self.generate_number_literal(lit, expression.1)
            }
            Expression::Literal(Literal::String(lit)) => Ok(self.generate_string_literal(lit)),
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
        }
    }

    fn generate_string_literal<'a: 'ctx>(&self, lit: &'a str) -> TypedValue<'ctx> {
        let string = self.context.const_string(lit.as_bytes(), false);

        let string_global = self.module.add_global(
            self.context.i8_type().array_type(lit.len() as u32),
            None,
            "",
        );

        string_global.set_initializer(&string);
        string_global.set_constant(true);

        let string_ptr = string_global.as_pointer_value();

        let str_len = self.core_types.isize.const_int(lit.len() as u64, false);

        TypedValue {
            val: self
                .core_types
                .str
                .const_named_struct(&[string_ptr.into(), str_len.into()])
                .into(),
            type_: Type::str,
        }
    }

    fn generate_number_literal<'a: 'ctx>(
        &self,
        lit: &'a str,
        span: Span,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        Ok(TypedValue {
            val: self
                .context
                .i32_type()
                .const_int_from_string(lit, StringRadix::Decimal)
                .ok_or(codegen::error::invalid_number(S(lit, span)))?
                .into(),
            type_: Type::i32,
        })
    }

    fn generate_function_call<'a: 'ctx>(
        &self,
        span: Span,
        scope: &mut Scope<'_, 'ctx>,
        fn_name: &'a str,
        arguments: &[S<Expression<'a>>],
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let function =
            scope
                .get_function(fn_name)
                .cloned()
                .ok_or(codegen::error::undefined_function(S(
                    fn_name,
                    span.with_len(fn_name.len()),
                )))?;

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
            .build_direct_call(function.function, &metadata_arguments, "")
            .unwrap();

        Ok(TypedValue {
            val: ret_val.try_as_basic_value().left().unwrap(),
            type_: function.signature.return_type,
        })
    }
}
