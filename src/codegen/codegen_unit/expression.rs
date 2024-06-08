use std::ops::Deref;

use crate::{
    codegen::{
        codegen_unit::CodegenUnit,
        error::CodegenError,
        scope::Scope,
        types::{Type, TypedValue},
    },
    error_handling::Spanned as S,
    parser::{Expression, Literal},
};

use inkwell::{types::StringRadix, values::BasicMetadataValueEnum};
use wutil::Span;

impl<'ctx> CodegenUnit<'ctx> {
    pub fn generate_expression<'a: 'ctx>(
        &self,
        expression: S<&Expression<'a>>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, CodegenError<'a>> {
        match expression.deref() {
            Expression::Identifier(ident) => scope
                .get_variable(ident)
                .cloned()
                .ok_or(CodegenError::UndefinedVariable(ident)),
            Expression::Literal(Literal::Number(lit)) => self.generate_number_literal(lit),
            Expression::Literal(Literal::String(lit)) => self.generate_string_literal(lit),
            Expression::BinaryOperator(a_expr, operator, b_expr) => {
                let a = self.generate_expression(a_expr.as_sref(), scope)?;
                let b = self.generate_expression(b_expr.as_sref(), scope)?;

                a.generate_operation(&self.builder, a_expr.1, *operator, S(b, b_expr.1))
            }
            Expression::CompoundExpression(_) => todo!(),
            Expression::FunctionCall(fn_name, arguments) => {
                self.generate_function_call(expression.1, scope, fn_name, arguments)
            }
        }
    }

    fn generate_string_literal<'a: 'ctx>(
        &self,
        lit: &'a str,
    ) -> Result<TypedValue<'ctx>, CodegenError<'a>> {
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

        Ok(TypedValue {
            val: self
                .core_types
                .str
                .const_named_struct(&[string_ptr.into(), str_len.into()])
                .into(),
            type_: Type::str,
        })
    }

    fn generate_number_literal<'a: 'ctx>(
        &self,
        lit: &'a str,
    ) -> Result<TypedValue<'ctx>, CodegenError<'a>> {
        Ok(TypedValue {
            val: self
                .context
                .i32_type()
                .const_int_from_string(lit, StringRadix::Decimal)
                .ok_or(CodegenError::InvalidNumber(lit))?
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
    ) -> Result<TypedValue<'ctx>, CodegenError<'a>> {
        let function = scope
            .get_function(fn_name)
            .cloned()
            .ok_or(CodegenError::UndefinedFunction(fn_name))?;

        if arguments.len() != function.params.len() {
            return Err(CodegenError::InvalidParamCount(
                span,
                function.params.len(),
                arguments.len(),
            ));
        }

        let mut metadata_arguments: Vec<BasicMetadataValueEnum> =
            Vec::with_capacity(arguments.len());

        for (i, arg) in arguments.iter().enumerate() {
            let arg = self.generate_expression(arg.as_sref(), scope)?;

            metadata_arguments.push(arg.val.into());

            let expected_type = &function.params[i];
            if expected_type != &arg.type_ {
                return Err(CodegenError::UnexpectedType(
                    arguments[i].1,
                    expected_type.to_string().into(),
                    arg.type_.to_string().into(),
                ));
            }
        }

        let _ret_val = self // TODO: return value
            .builder
            .build_direct_call(function.function.clone(), &metadata_arguments, "")
            .unwrap();

        Ok(TypedValue {
            val: self.context.i32_type().const_zero().into(),
            type_: Type::i32,
        })
    }
}
