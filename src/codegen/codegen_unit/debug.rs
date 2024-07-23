use inkwell::{
    debug_info::{
        AsDIScope, DIBasicType, DICompileUnit, DIFlagsConstants, DIScope, DIType, DebugInfoBuilder,
    },
    module::Module as LlvmModule,
};

use crate::codegen::{types::Type, CodegenContext};

struct DebugPrimitives<'ctx> {
    pub i32: DIBasicType<'ctx>,
    pub str: DIBasicType<'ctx>,
    pub unit: DIBasicType<'ctx>,
    pub bool: DIBasicType<'ctx>,
}

pub struct DebugContext<'ctx> {
    pub builder: DebugInfoBuilder<'ctx>,
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

        let (builder, cu) = module.create_debug_info_builder(
            true,
            inkwell::debug_info::DWARFSourceLanguage::C,
            file_name,
            directory,
            "wlab",
            false, // OPT_LEVEL
            "",
            0,
            "",
            inkwell::debug_info::DWARFEmissionKind::Full,
            0,
            false,
            false,
            "",
            "",
        );

        let primitives = DebugPrimitives::new(c, &builder);

        Self {
            builder,
            cu,
            scope: cu.as_debug_info_scope(),
            primitives,
        }
    }

    pub fn get_type(&self, type_: &Type) -> DIType<'ctx> {
        match type_ {
            Type::i32 => self.primitives.i32.as_type(),
            Type::str => self.primitives.str.as_type(),
            Type::unit => self.primitives.unit.as_type(),
            Type::bool => self.primitives.bool.as_type(),
        }
    }
}

impl<'ctx> DebugPrimitives<'ctx> {
    fn new(cc: &CodegenContext<'ctx>, builder: &DebugInfoBuilder<'ctx>) -> Self {
        let i32 = builder
            .create_basic_type("i32", 32, 0, DIFlagsConstants::PRIVATE)
            .unwrap();
        let str = builder
            .create_basic_type(
                "str",
                (2 * cc.target.get_target_data().get_pointer_byte_size(None) * 8) as u64,
                0,
                DIFlagsConstants::PRIVATE,
            )
            .unwrap();
        let unit = builder
            .create_basic_type("unit", 0, 0, DIFlagsConstants::PRIVATE)
            .unwrap();
        let bool = builder
            .create_basic_type("bool", 1, 0, DIFlagsConstants::PRIVATE)
            .unwrap();

        Self {
            i32,
            str,
            unit,
            bool,
        }
    }
}
