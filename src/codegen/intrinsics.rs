use inkwell::values::{IntValue, StructValue};
use wutil::Span;

use crate::{
    codegen::{self, scope::FunctionInfo, types::Type, CodegenUnit},
    error_handling::{Diagnostic, Spanned as S},
    parser::Function,
};

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn add_intrinsic(
        &self,
        function: &S<Function>,
        function_info: &FunctionInfo,
        params: &[(&str, Type)],
        intrinsic: S<&str>,
    ) -> Result<(), Diagnostic> {
        if !function.body.body.is_empty() {
            return Err(codegen::error::non_empty_intrinsic(function.body.1));
        }

        match *intrinsic {
            "write" => add_write(function.1, self, function_info, params),
            "exit" => add_exit(function.1, self, function_info, params),
            _ => Err(codegen::error::invalid_intrinsic(intrinsic)),
        }
    }
}

fn add_write(
    function_span: Span,
    unit: &CodegenUnit<'_, '_>,
    function_info: &FunctionInfo,
    params: &[(&str, Type)],
) -> Result<(), Diagnostic> {
    if !matches!(params, [(_, Type::i32), (_, Type::str)]) {
        return Err(codegen::error::invalid_intrinsic_params(
            function_span,
            "(i32, str)",
        ));
    }

    if function_info.signature.return_type != Type::unit {
        return Err(codegen::error::invalid_intrinsic_ret_type(
            function_span,
            &Type::unit,
        ));
    }

    let i64 = unit.c.context.i64_type();

    let function = function_info.function;

    let main_block = unit.c.context.append_basic_block(function, "");
    unit.builder.position_at_end(main_block);

    let syscall_type = i64.fn_type(
        &[
            i64.into(),
            i64.into(),
            unit.c.context.i8_type().ptr_type(Default::default()).into(),
            unit.c.core_types.isize.into(),
        ],
        false,
    );

    let syscall = unit.c.context.create_inline_asm(
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
            IntValue::try_from(function.get_nth_param(0).unwrap()).unwrap(),
            i64,
            "",
        )
        .unwrap();

    let data_ptr = unit
        .builder
        .build_extract_value(
            StructValue::try_from(function.get_nth_param(1).unwrap()).unwrap(),
            0,
            "",
        )
        .unwrap();

    let str_len = unit
        .builder
        .build_extract_value(
            StructValue::try_from(function.get_nth_param(1).unwrap()).unwrap(),
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

    let zero = unit.c.core_types.unit.const_zero();
    unit.builder.build_return(Some(&zero)).unwrap();

    Ok(())
}

fn add_exit(
    function_span: Span,
    unit: &CodegenUnit<'_, '_>,
    function_info: &FunctionInfo,
    params: &[(&str, Type)],
) -> Result<(), Diagnostic> {
    if !matches!(params, [(_, Type::i32)]) {
        return Err(codegen::error::invalid_intrinsic_params(
            function_span,
            "(i32)",
        ));
    }

    if function_info.signature.return_type != Type::unit {
        return Err(codegen::error::invalid_intrinsic_ret_type(
            function_span,
            &Type::unit,
        ));
    }

    let i64 = unit.c.context.i64_type();

    let function = function_info.function;

    let main_block = unit.c.context.append_basic_block(function, "");
    unit.builder.position_at_end(main_block);

    let syscall_type = i64.fn_type(&[i64.into(), i64.into()], false);

    let syscall = unit.c.context.create_inline_asm(
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
            IntValue::try_from(function.get_nth_param(0).unwrap()).unwrap(),
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

    let zero = unit.c.core_types.unit.const_zero();
    unit.builder.build_return(Some(&zero)).unwrap();

    Ok(())
}
