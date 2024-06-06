use std::mem;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, StringRadix, StructType},
    values::{BasicMetadataValueEnum, IntValue},
};

use crate::parser::{Expression, Statement};

use self::{
    error::CodegenError,
    scope::{FunctionInfo, Scope},
    types::{Type, TypedValue},
};

mod error;
mod scope;

mod types;

struct CoreTypes<'ctx> {
    unit: StructType<'ctx>,
}

impl<'ctx> CoreTypes<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            unit: context.struct_type(&[], false),
        }
    }
}

struct CodegenUnit<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    core_types: CoreTypes<'ctx>,
    module: Module<'ctx>,
}

impl<'ctx> CodegenUnit<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("my_module"),
            builder: context.create_builder(),
            core_types: CoreTypes::new(context),
        }
    }

    pub fn generate_expression<'a: 'ctx>(
        &self,
        expression: &Expression<'a>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, CodegenError<'a>> {
        match expression {
            Expression::Identifier(ident) => scope
                .get_variable(ident)
                .cloned()
                .ok_or(CodegenError::UndefinedVariable(ident)),
            Expression::Literal(lit) => Ok(TypedValue {
                val: self
                    .context
                    .i32_type()
                    .const_int_from_string(lit, StringRadix::Decimal)
                    .ok_or(CodegenError::InvalidNumber(lit))?
                    .into(),
                type_: Type::i32,
            }),
            Expression::BinaryOperator(a, operator, b) => {
                let a = self.generate_expression(a, scope)?;
                let b = self.generate_expression(b, scope)?;

                a.generate_operation(&self.builder, *operator, b)
            }
            Expression::CompoundExpression(_) => todo!(),
            Expression::FunctionCall(fn_name, arguments) => {
                let function = scope
                    .get_function(fn_name)
                    .cloned()
                    .ok_or(CodegenError::UndefinedFunction(fn_name))?;

                if arguments.len() != function.num_params {
                    return Err(CodegenError::InvalidParameters(
                        fn_name,
                        function.num_params,
                        arguments.len(),
                    ));
                }

                let arguments: Result<Vec<_>, _> = arguments
                    .iter()
                    .map(|e| self.generate_expression(e, scope))
                    .map(|v| Ok(v?.val.into()))
                    .collect();

                let arguments: Vec<BasicMetadataValueEnum> = arguments?;

                let _ret_val = self // TODO: return value
                    .builder
                    .build_direct_call(function.function.clone(), &arguments, "")
                    .unwrap();

                Ok(TypedValue {
                    val: self.context.i32_type().const_zero().into(),
                    type_: Type::i32,
                })
            }
        }
    }

    pub fn generate_statement<'a: 'ctx>(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        statement: &Statement<'a>,
    ) -> Result<(), CodegenError<'a>> {
        match statement {
            Statement::Expression(expr) => mem::drop(self.generate_expression(expr, scope)?),
            Statement::Let(varname, val) => {
                let val = self.generate_expression(val, scope)?;
                scope.create_variable(varname, val);
            }
            Statement::Assign(_, _) => todo!(),
            Statement::Function(_, _, _) => todo!(),
        }
        Ok(())
    }

    pub fn generate_function<'a: 'ctx>(
        &self,
        fn_name: &str,
        params: &[(&'a str, &'a str)],
        body: &[Statement<'a>],
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<(), CodegenError<'a>> {
        let params: Result<Vec<(&'a str, Type)>, _> = params
            .iter()
            .map(|(n, t)| Ok((*n, Type::new(t)?)))
            .collect();
        let params = params?;

        let llvm_param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|(_, type_)| type_.get_llvm_type(self.context).into())
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
                num_params: params.len(),
                function,
            },
        );

        Ok(())
    }
}

pub fn generate_code<'a>(ast: &[Statement<'a>]) -> Result<(), CodegenError<'a>> {
    let context = Context::create();
    let generator = CodegenUnit::new(&context);
    let mut scope = Scope::new_global();

    for s in ast {
        let Statement::Function(fn_name, params, body) = s else {
            todo!()
        };

        generator.generate_function(fn_name, params, body, &mut scope)?;
    }

    println!("{}", generator.module.to_string());

    Ok(())
}
