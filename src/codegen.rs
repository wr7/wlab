use inkwell::{
    builder::{Builder, BuilderError},
    context::Context,
    module::Module,
    types::StructType,
    values::IntValue,
};

use crate::parser::{Expression, Statement};

struct CoreTypes<'a> {
    unit: StructType<'a>,
}

impl<'a> CoreTypes<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            unit: context.struct_type(&[], false),
        }
    }
}

struct CodeGenerator<'a> {
    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
    types: CoreTypes<'a>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            context,
            module: context.create_module("my_module"),
            builder: context.create_builder(),
            types: CoreTypes::new(context),
        }
    }

    pub fn generate_expression(&self, expression: &Expression) -> Result<IntValue, BuilderError> {
        match expression {
            Expression::Identifier(_) => todo!(),
            Expression::BinaryOperator(a, operator, b) => {
                let a = self.generate_expression(a)?;
                let b = self.generate_expression(b)?;

                match operator {
                    crate::parser::OpCode::Plus => self.builder.build_int_add(a, b, "add"),
                    crate::parser::OpCode::Minus => todo!(),
                    crate::parser::OpCode::Asterisk => todo!(),
                    crate::parser::OpCode::Slash => todo!(),
                }
            }
            Expression::CompoundExpression(_) => todo!(),
        }
    }

    pub fn generate_statement(&self, statement: &Statement) -> Result<(), BuilderError> {
        match statement {
            Statement::Expression(_) => todo!(),
            Statement::Let(varname, val) => {
                let val = self.generate_expression(val)?;
                // TODO: assign to variable
            }
            Statement::Assign(_, _) => todo!(),
            Statement::Function(_, _) => todo!(),
        }
        Ok(())
    }

    pub fn generate_function(&self, fn_name: &str, body: &[Statement]) {
        let main = self
            .module
            .add_function(fn_name, self.types.unit.fn_type(&[], false), None);

        let main_block = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(main_block);

        let zero = self.types.unit.const_zero();

        for statement in body {
            todo!()
        }

        self.builder.build_return(Some(&zero)).unwrap();
    }
}

pub fn generate_code(ast: &[Statement]) {
    let context = Context::create();
    let generator = CodeGenerator::new(&context);

    for s in ast {
        let Statement::Function(fn_name, body) = s else {
            todo!()
        };

        generator.generate_function(fn_name, body);
    }

    println!("{}", generator.module.to_string());
}
