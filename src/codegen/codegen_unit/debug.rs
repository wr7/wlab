use wllvm::{
    debug_info::{
        DIBasicType, DIBuilder, DICompileUnit, DIFlags, DIScope, DIType, SourceLanguage,
        TypeEncoding,
    },
    target::OptLevel,
    Module as LlvmModule,
};

use crate::codegen::{types::Type, CodegenContext};

struct DebugPrimitives<'ctx> {
    pub i32: DIBasicType<'ctx>,
    pub str: DIBasicType<'ctx>,
    pub unit: DIBasicType<'ctx>,
    pub bool: DIBasicType<'ctx>,
}

pub struct DebugContext<'ctx> {
    pub builder: DIBuilder<'ctx>,
    pub cu: DICompileUnit<'ctx>,
    pub scope: DIScope<'ctx>,
    primitives: DebugPrimitives<'ctx>,
}

impl<'ctx> DebugContext<'ctx> {
    pub(super) fn new(
        c: &CodegenContext<'ctx>,
        module: &LlvmModule<'ctx>,
        file_path: &str,
    ) -> Self {
        let (directory, file_name) = file_path.rsplit_once('/').unwrap_or((".", file_path));
        let directory = directory.trim_end_matches('/');

        let builder = DIBuilder::new(module);
        let file = builder.file(file_name, directory);

        let cu = builder
            .build_compile_unit(
                file,
                SourceLanguage::C,
                "wlab",
                c.params.opt_level != OptLevel::None,
                "", // TODO: add flags
                0,
            )
            .debug_info_for_profiling(false)
            .build();

        let primitives = DebugPrimitives::new(c, &builder);

        Self {
            builder,
            cu,
            scope: *cu,
            primitives,
        }
    }

    pub fn get_type(&self, type_: &Type) -> DIType<'ctx> {
        match type_ {
            Type::i32 => *self.primitives.i32,
            Type::str => *self.primitives.str,
            Type::unit => *self.primitives.unit,
            Type::bool => *self.primitives.bool,
        }
    }
}

impl<'ctx> DebugPrimitives<'ctx> {
    fn new(cc: &CodegenContext<'ctx>, builder: &DIBuilder<'ctx>) -> Self {
        let i32 = builder.basic_type("i32", 32, Some(TypeEncoding::signed), DIFlags::Private);

        let str = builder.basic_type(
            "str",
            u64::from(2 * cc.target_data.ptr_size() * 8),
            None,
            DIFlags::Private,
        );
        let unit = builder.basic_type("unit", 0, None, DIFlags::Private);
        let bool = builder.basic_type("bool", 1, Some(TypeEncoding::boolean), DIFlags::Private);

        Self {
            i32,
            str,
            unit,
            bool,
        }
    }
}
