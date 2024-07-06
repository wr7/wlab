use inkwell::{
    context::Context,
    targets::TargetMachine,
    types::{IntType, StructType},
    AddressSpace,
};

mod codegen_context;
mod codegen_unit;

pub use codegen_context::CodegenContext;

mod error;
mod intrinsics;
mod namestore;
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
