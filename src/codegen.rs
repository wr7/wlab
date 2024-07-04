use inkwell::{
    context::Context,
    targets::TargetMachine,
    types::{IntType, StructType},
    AddressSpace,
};

use crate::{error_handling::Diagnostic, parser::Module};

use self::codegen_context::CodegenContext;

mod codegen_context;
mod codegen_unit;

mod error;
mod intrinsics;
mod scope;
mod types;

use codegen_unit::CodegenUnit;

struct CoreTypes<'ctx> {
    unit: StructType<'ctx>,
    bool: IntType<'ctx>,
    i32: IntType<'ctx>,
    isize: IntType<'ctx>,
    str: StructType<'ctx>,
}

impl<'ctx> CoreTypes<'ctx> {
    pub fn new(context: &'ctx Context, target: &TargetMachine) -> Self {
        let target_data = target.get_target_data();

        let bool = context.bool_type();
        let isize = context.ptr_sized_int_type(&target_data, None);
        let i32 = context.i32_type();

        Self {
            unit: context.struct_type(&[], false),
            bool,
            i32,
            isize,
            str: context.struct_type(
                &[i32.ptr_type(AddressSpace::default()).into(), isize.into()],
                false,
            ),
        }
    }
}

pub fn generate_code(crates: &[Module]) -> Result<(), (usize, Diagnostic)> {
    let context = Context::create();
    let mut codegen_context = CodegenContext::new(&context);

    for (i, crate_) in crates.iter().enumerate() {
        codegen_context.generate_code(crate_).map_err(|e| (i, e))?;
    }

    Ok(())
}
