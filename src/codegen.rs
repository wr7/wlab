use inkwell::{
    context::Context,
    targets::TargetMachine,
    types::{IntType, StructType},
    AddressSpace,
};

use crate::parser::Statement;

use self::{error::CodegenError, scope::Scope};

mod codegen_unit;
mod error;
mod intrinsics;
mod scope;

mod types;

pub(self) use codegen_unit::CodegenUnit;

struct CoreTypes<'ctx> {
    unit: StructType<'ctx>,
    i32: IntType<'ctx>,
    isize: IntType<'ctx>,
    str: StructType<'ctx>,
}

impl<'ctx> CoreTypes<'ctx> {
    pub fn new(context: &'ctx Context, target: &TargetMachine) -> Self {
        let target_data = target.get_target_data();

        let isize = context.ptr_sized_int_type(&target_data, None);
        let i32 = context.i32_type();

        Self {
            unit: context.struct_type(&[], false),
            i32,
            isize,
            str: context.struct_type(
                &[i32.ptr_type(AddressSpace::default()).into(), isize.into()],
                false,
            ),
        }
    }
}

pub fn generate_code<'a>(ast: &[Statement<'a>]) -> Result<(), CodegenError<'a>> {
    let context = Context::create();
    let generator = CodegenUnit::new(&context);
    let mut scope = Scope::new_global();

    intrinsics::add_intrinsics(&generator, &mut scope);

    for s in ast {
        let Statement::Function(fn_name, params, body) = s else {
            todo!()
        };

        generator.generate_function(fn_name, params, body, &mut scope)?;
    }

    println!("{}", generator.module.to_string());

    Ok(())
}
