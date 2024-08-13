use std::cell::Cell;

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
    pub str: DIBasicType<'ctx>,
    pub unit: DIBasicType<'ctx>,
    pub bool: DIBasicType<'ctx>,
    int_types: Cell<Vec<(u32, DIBasicType<'ctx>)>>,
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
        match *type_ {
            Type::i(n) => *self.int(n),
            Type::str => *self.primitives.str,
            Type::unit => *self.primitives.unit,
            Type::bool => *self.primitives.bool,
        }
    }

    fn int(&self, size: u32) -> DIBasicType<'ctx> {
        let mut int_types = self.primitives.int_types.take();

        let ret_val = match int_types.binary_search_by_key(&size, |(k, _)| *k) {
            Ok(idx) => int_types[idx].1,
            Err(idx) => {
                let type_ = self.builder.basic_type(
                    &format!("i{size}"),
                    size.into(),
                    Some(TypeEncoding::signed),
                    DIFlags::Private,
                );

                int_types.insert(idx, (size, type_));

                type_
            }
        };

        self.primitives.int_types.set(int_types);

        ret_val
    }
}

impl<'ctx> DebugPrimitives<'ctx> {
    fn new(cc: &CodegenContext<'ctx>, builder: &DIBuilder<'ctx>) -> Self {
        let str = builder.basic_type(
            "str",
            u64::from(2 * cc.target_data.ptr_size() * 8),
            None,
            DIFlags::Private,
        );
        let unit = builder.basic_type("unit", 0, None, DIFlags::Private);
        let bool = builder.basic_type("bool", 1, Some(TypeEncoding::boolean), DIFlags::Private);

        Self {
            str,
            unit,
            bool,
            int_types: Vec::new().into(),
        }
    }
}
