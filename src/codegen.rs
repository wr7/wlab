use wllvm::{
    target::TargetData,
    type_::{IntType, StructType},
    Context,
};

mod codegen_context;
mod codegen_unit;

pub use codegen_context::CodegenContext;

mod error;
mod intrinsics;
mod namestore;
mod scope;
mod types;
mod values;

use codegen_unit::CodegenUnit;

struct CoreTypes<'ctx> {
    unit: StructType<'ctx>,
    bool: IntType<'ctx>,
    isize: IntType<'ctx>,
    str: StructType<'ctx>,
}

impl<'ctx> CoreTypes<'ctx> {
    pub fn new(context: &'ctx Context, target_data: &TargetData) -> Self {
        let bool = context.int_type(1);
        let isize = context.ptr_sized_int_type(target_data);

        Self {
            unit: context.struct_type(&[], false),
            bool,
            isize,
            str: context.struct_type(&[*context.ptr_type(), *isize], false),
        }
    }
}
