use std::borrow::Borrow;

use inkwell::{
    builder::{Builder, BuilderError},
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, IntType, StructType},
    values::IntValue,
};

use crate::parser::{Expression, Statement};

use self::scope::Scope;

mod scope;

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
    global_scope: Scope<'ctx>,
}

impl<'ctx> CodegenUnit<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("my_module"),
            builder: context.create_builder(),
            core_types: CoreTypes::new(context),
            global_scope: Scope::new(),
        }
    }

    pub fn generate_expression(
        &self,
        expression: &Expression,
        scope: &mut Scope<'ctx>,
    ) -> Result<IntValue<'ctx>, BuilderError> {
        match expression {
            // TODO: handle undefined ident
            Expression::Identifier(ident) => Ok(scope.get_variable(ident).unwrap()),
            Expression::BinaryOperator(a, operator, b) => {
                let a = self.generate_expression(a, scope)?;
                let b = self.generate_expression(b, scope)?;

                match operator {
                    crate::parser::OpCode::Plus => self.builder.build_int_add(a, b, ""),
                    crate::parser::OpCode::Minus => todo!(),
                    crate::parser::OpCode::Asterisk => todo!(),
                    crate::parser::OpCode::Slash => todo!(),
                }
            }
            Expression::CompoundExpression(_) => todo!(),
        }
    }

    pub fn generate_statement(
        &self,
        scope: &mut Scope<'ctx>,
        statement: &Statement<'ctx>,
    ) -> Result<(), BuilderError> {
        match statement {
            Statement::Expression(_) => todo!(),
            Statement::Let(varname, val) => {
                let val = self.generate_expression(val, scope)?;
                scope.create_variable(varname, val);
            }
            Statement::Assign(_, _) => todo!(),
            Statement::Function(_, _, _) => todo!(),
        }
        Ok(())
    }

    pub fn generate_function(
        &self,
        fn_name: &str,
        params: &[&str],
        body: &[Statement<'ctx>],
    ) -> Result<(), BuilderError> {
        let fn_type_params: Vec<BasicMetadataTypeEnum<'ctx>> =
            std::iter::repeat(BasicMetadataTypeEnum::IntType(self.context.i32_type()))
                .take(params.len())
                .collect();

        let function = self.module.add_function(
            fn_name,
            self.core_types.unit.fn_type(&fn_type_params, false),
            None,
        );

        let main_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(main_block);

        let zero = self.core_types.unit.const_zero();

        let mut scope = Scope::new().with_params(params, &function);

        for statement in body {
            self.generate_statement(&mut scope, statement)?;
        }

        self.builder.build_return(Some(&zero)).unwrap();

        Ok(())
    }
}

pub fn generate_code(ast: &[Statement]) {
    let context = Context::create();
    let generator = CodegenUnit::new(&context);

    for s in ast {
        let Statement::Function(fn_name, params, body) = s else {
            todo!()
        };

        generator.generate_function(fn_name, params, body).unwrap();
    }

    println!("{}", generator.module.to_string());
}
