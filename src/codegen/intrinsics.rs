use wllvm::{
    type_::AsmDialect,
    value::{IntValue, StructValue},
};
use wutil::Span;

use crate::{
    codegen::{self, namestore::FunctionInfo, types::Type, CodegenUnit},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast,
    util,
};

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn add_intrinsic(
        &self,
        function: &S<ast::Function>,
        function_info: &FunctionInfo,
        params: &[(S<&str>, Type)],
        intrinsic: S<&str>,
    ) -> Result<(), Diagnostic> {
        let (line_no, col_no) = util::line_and_col(self.source, function.body.1.start);
        let dbg_location = self.c.context.debug_location(
            line_no as u32,
            col_no as u32,
            self.debug_context.scope,
            None,
        );

        self.builder.set_debug_location(dbg_location);

        if !function.body.body.is_empty() {
            return Err(codegen::error::non_empty_intrinsic(function.body.1));
        }

        match *intrinsic {
            "write" => add_write(function.params.1, self, function_info, params),
            "exit" => add_exit(function.params.1, self, function_info, params),
            _ => Err(codegen::error::invalid_intrinsic(intrinsic)),
        }
    }
}

fn add_write(
    params_span: Span,
    unit: &CodegenUnit<'_, '_>,
    function_info: &FunctionInfo,
    params: &[(S<&str>, Type)],
) -> Result<(), Diagnostic> {
    if !matches!(params, [(_, Type::i(32)), (_, Type::str)]) {
        return Err(codegen::error::invalid_intrinsic_params(
            params_span,
            "(i32, str)",
        ));
    }

    if function_info.signature.return_type != Type::unit {
        return Err(codegen::error::invalid_intrinsic_ret_type(
            params_span,
            &Type::unit,
        ));
    }

    let i64 = unit.c.context.int_type(64);

    let function = function_info.function;

    let main_block = function.add_basic_block(c"");
    unit.builder.position_at_end(main_block);

    let syscall_type = unit.c.context.fn_type(
        *i64,
        &[
            *i64,
            *i64,
            *unit.c.context.ptr_type(),
            *unit.c.core_types.isize,
        ],
        false,
    );

    let syscall = syscall_type.inline_asm(
        "syscall",
        "=r,{rax},{rdi},{rsi},{rdx}",
        true,
        false,
        AsmDialect::ATT,
        false,
    );

    // params //

    let fd = unit.builder.build_zext(
        IntValue::try_from(function.param(0).unwrap()).unwrap(),
        i64,
        c"",
    );

    let data_ptr = unit
        .builder
        .build_extract_value(
            StructValue::try_from(function.param(1).unwrap()).unwrap(),
            0,
            c"",
        )
        .unwrap();

    let str_len = unit
        .builder
        .build_extract_value(
            StructValue::try_from(function.param(1).unwrap()).unwrap(),
            1,
            c"",
        )
        .unwrap();

    // do call //

    unit.builder.build_ptr_call(
        syscall_type,
        syscall,
        &[*i64.const_(1, false), *fd, data_ptr, str_len],
        c"",
    );

    let zero = unit.c.core_types.unit.const_(&[]);
    unit.builder.build_ret(*zero);

    Ok(())
}

fn add_exit(
    params_span: Span,
    unit: &CodegenUnit<'_, '_>,
    function_info: &FunctionInfo,
    params: &[(S<&str>, Type)],
) -> Result<(), Diagnostic> {
    if !matches!(params, [(_, Type::i(32))]) {
        return Err(codegen::error::invalid_intrinsic_params(
            params_span,
            "(i32)",
        ));
    }

    if function_info.signature.return_type != Type::never {
        return Err(codegen::error::invalid_intrinsic_ret_type(
            params_span,
            &Type::never,
        ));
    }

    let i64 = unit.c.context.int_type(64);

    let function = function_info.function;

    let main_block = function.add_basic_block(c"");
    unit.builder.position_at_end(main_block);

    let syscall_type = unit.c.context.fn_type(*i64, &[*i64, *i64], false);

    let syscall = syscall_type.inline_asm(
        "syscall",
        "=r,{rax},{rdi}",
        true,
        false,
        AsmDialect::ATT,
        false,
    );

    // params //

    let exit_code = unit.builder.build_zext(
        IntValue::try_from(function.param(0).unwrap()).unwrap(),
        i64,
        c"",
    );

    // do call //

    unit.builder.build_ptr_call(
        syscall_type,
        syscall,
        &[*i64.const_(60, false), *exit_code],
        c"",
    );

    let zero = unit.c.core_types.unit.const_(&[]);
    unit.builder.build_ret(*zero);

    Ok(())
}
