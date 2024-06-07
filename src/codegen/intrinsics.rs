use inkwell::values::{IntValue, StructValue};

use super::{
    scope::{FunctionInfo, Scope},
    types::Type,
    CodegenUnit,
};

pub fn add_intrinsics<'ctx>(unit: &CodegenUnit<'ctx>, scope: &mut Scope<'_, 'ctx>) {
    add_write(unit, scope);
    add_exit(unit, scope);
}

fn add_write<'ctx>(unit: &CodegenUnit<'ctx>, scope: &mut Scope<'_, 'ctx>) {
    let i64 = unit.context.i64_type();
    let i32 = unit.core_types.i32;
    let str = unit.core_types.str;

    let write = unit.module.add_function(
        "write",
        unit.core_types
            .unit
            .fn_type(&[i32.into(), str.into()], false),
        None,
    );

    let main_block = unit.context.append_basic_block(write, "");
    unit.builder.position_at_end(main_block);

    let syscall_type = i64.fn_type(
        &[
            i64.into(),
            i64.into(),
            unit.context.i8_type().ptr_type(Default::default()).into(),
            unit.core_types.isize.into(),
        ],
        false,
    );

    let syscall = unit.context.create_inline_asm(
        syscall_type,
        "syscall".into(),
        "=r,{rax},{rdi},{rsi},{rdx}".into(),
        true,
        false,
        None,
        false,
    );

    // params //

    let fd = unit
        .builder
        .build_int_z_extend(
            IntValue::try_from(write.get_nth_param(0).unwrap()).unwrap(),
            i64,
            "",
        )
        .unwrap();

    let data_ptr = unit
        .builder
        .build_extract_value(
            StructValue::try_from(write.get_nth_param(1).unwrap()).unwrap(),
            0,
            "",
        )
        .unwrap();

    let str_len = unit
        .builder
        .build_extract_value(
            StructValue::try_from(write.get_nth_param(1).unwrap()).unwrap(),
            1,
            "",
        )
        .unwrap();

    // do call //

    unit.builder
        .build_indirect_call(
            syscall_type,
            syscall,
            &[
                i64.const_int(1, false).into(),
                fd.into(),
                data_ptr.into(),
                str_len.into(),
            ],
            "",
        )
        .unwrap();

    let zero = unit.core_types.unit.const_zero();
    unit.builder.build_return(Some(&zero)).unwrap();

    scope.create_function(
        "write",
        FunctionInfo {
            params: vec![Type::i32, Type::str],
            function: write,
        },
    );
}

fn add_exit<'ctx>(unit: &CodegenUnit<'ctx>, scope: &mut Scope<'_, 'ctx>) {
    let i64 = unit.context.i64_type();
    let i32 = unit.core_types.i32;

    let exit = unit.module.add_function(
        "exit",
        unit.core_types.unit.fn_type(&[i32.into()], false),
        None,
    );

    let main_block = unit.context.append_basic_block(exit, "");
    unit.builder.position_at_end(main_block);

    let syscall_type = i64.fn_type(&[i64.into(), i64.into()], false);

    let syscall = unit.context.create_inline_asm(
        syscall_type,
        "syscall".into(),
        "=r,{rax},{rdi}".into(),
        true,
        false,
        None,
        false,
    );

    // params //

    let exit_code = unit
        .builder
        .build_int_z_extend(
            IntValue::try_from(exit.get_nth_param(0).unwrap()).unwrap(),
            i64,
            "",
        )
        .unwrap();

    // do call //

    unit.builder
        .build_indirect_call(
            syscall_type,
            syscall,
            &[i64.const_int(60, false).into(), exit_code.into()],
            "",
        )
        .unwrap();

    let zero = unit.core_types.unit.const_zero();
    unit.builder.build_return(Some(&zero)).unwrap();

    scope.create_function(
        "exit",
        FunctionInfo {
            params: vec![Type::i32],
            function: exit,
        },
    );
}
