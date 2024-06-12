use std::path::Path;

use inkwell::{
    context::Context,
    targets::TargetMachine,
    types::{IntType, StructType},
    AddressSpace,
};

use crate::{
    error_handling::{Diagnostic, Spanned as S},
    parser::Statement,
};

use self::scope::Scope;

mod codegen_unit;
mod error;
mod intrinsics;
mod scope;

mod types;

use codegen_unit::CodegenUnit;

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

pub fn generate_code(ast: &[S<Statement<'_>>]) -> Result<(), Diagnostic> {
    let context = Context::create();
    let generator = CodegenUnit::new(&context);
    let mut scope = Scope::new_global();

    intrinsics::add_intrinsics(&generator, &mut scope);

    for s in ast {
        let Statement::Function(fn_name, params, body) = &**s else {
            todo!()
        };

        generator.generate_function(fn_name, params, body, &mut scope)?;
    }

    let llvm_ir = generator.module.to_string();

    std::fs::write("./a.llvm", llvm_ir).unwrap();

    generator
        .target
        .write_to_file(
            &generator.module,
            inkwell::targets::FileType::Object,
            Path::new("./a.o"),
        )
        .unwrap();

    generator
        .target
        .write_to_file(
            &generator.module,
            inkwell::targets::FileType::Assembly,
            Path::new("./a.asm"),
        )
        .unwrap();

    Ok(())
}
