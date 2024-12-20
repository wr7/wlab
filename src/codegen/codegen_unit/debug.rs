use wllvm::{
    debug_info::{
        DIBasicType, DIBuilder, DICompileUnit, DIFile, DIFlags, DIType, SourceLanguage,
        TypeEncoding,
    },
    target::OptLevel,
    type_::StructType,
    Module as LlvmModule,
};

use crate::{
    codegen::{namestore::FieldInfo, types::Type, CodegenContext},
    util::{BinarySearchMap, SharedBinarySearchMap},
};

use super::CodegenUnit;

struct DebugPrimitives<'ctx> {
    pub str: DIBasicType<'ctx>,
    pub unit: DIBasicType<'ctx>,
    pub bool: DIBasicType<'ctx>,
    pub never: DIBasicType<'ctx>,
    int_types: SharedBinarySearchMap<u32, DIBasicType<'ctx>>,
}

pub struct DebugContext<'ctx> {
    pub builder: DIBuilder<'ctx>,
    pub cu: DICompileUnit<'ctx>,
    primitives: DebugPrimitives<'ctx>,
    structs: SharedBinarySearchMap<String, DIType<'ctx>>,
    files: SharedBinarySearchMap<usize, DIFile<'ctx>>,
}

impl<'ctx> DebugContext<'ctx> {
    pub(super) fn new(c: &CodegenContext<'ctx>, module: &LlvmModule<'ctx>, file_no: usize) -> Self {
        let file_path = &c.files[file_no];
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

        let mut files = BinarySearchMap::new();
        files.insert_at(0, file_no, file);

        Self {
            builder,
            cu,
            primitives,
            structs: BinarySearchMap::new().into(),
            files: files.into(),
        }
    }

    pub fn get_file(&self, cc: &CodegenContext<'ctx>, file_no: usize) -> DIFile<'ctx> {
        self.files.get_or_insert_with(&file_no, || {
            let file_path = &cc.files[file_no];
            let (directory, file_name) = file_path.rsplit_once('/').unwrap_or((".", file_path));
            let directory = directory.trim_end_matches('/');

            self.builder.file(&file_name, &directory)
        })
    }

    pub fn get_type(&self, type_: &Type, cu: &CodegenUnit<'_, 'ctx>) -> DIType<'ctx> {
        match *type_ {
            Type::i(n) => *self.int(n),
            Type::str => *self.primitives.str,
            Type::unit => *self.primitives.unit,
            Type::bool => *self.primitives.bool,
            Type::never => *self.primitives.never,
            Type::Struct { ref path } => {
                if let Some(ty) = self.structs.get(path) {
                    return ty;
                }

                let struct_info =
                    cu.c.name_store
                        .get_item_from_string(path)
                        .unwrap()
                        .as_struct()
                        .unwrap();

                let file = self.get_file(cu.c, struct_info.file_no);

                let Some(llvm_type) = type_.llvm_type(cu.c) else {
                    let di_type = *self.builder.basic_type(path, 0, None, DIFlags::Private);

                    self.structs.insert(path.clone(), di_type).unwrap();

                    return di_type;
                };

                let Ok(llvm_ty) = StructType::try_from(llvm_type) else {
                    unreachable!()
                };

                let mut member_types = Vec::new();

                for (i, FieldInfo { name, ty, line_no }) in struct_info.fields.iter().enumerate() {
                    let llvm_field_ty = ty.llvm_type(cu.c).unwrap();
                    let size = llvm_field_ty.size_bits(&cu.c.target_data);
                    let align = llvm_field_ty.alignment(&cu.c.target_data);

                    let dbg_field_ty = ty.get_dwarf_type(cu);

                    member_types.push(self.builder.member_type(
                        *self.cu,
                        name,
                        file,
                        *line_no,
                        size,
                        align * 8,
                        llvm_ty.offset_of(&cu.c.target_data, i as u32) * 8,
                        DIFlags::Zero,
                        dbg_field_ty,
                    ));
                }

                let size_bits = llvm_ty.size_bits(&cu.c.target_data);
                let align_bits = llvm_ty.alignment(&cu.c.target_data) * 8;

                let struct_type = self.builder.struct_type(
                    *self.cu,
                    path,
                    file,
                    struct_info.line_no,
                    size_bits,
                    align_bits,
                    DIFlags::Private,
                    None,
                    &member_types,
                    None,
                    None,
                    "",
                );

                self.structs.insert(path.clone(), *struct_type).unwrap();

                *struct_type
            }
        }
    }

    fn int(&self, size: u32) -> DIBasicType<'ctx> {
        self.primitives.int_types.get_or_insert_with(&size, || {
            self.builder.basic_type(
                &format!("i{size}"),
                size.into(),
                Some(TypeEncoding::signed),
                DIFlags::Private,
            )
        })
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
        let never = builder.basic_type("!", 0, None, DIFlags::Private);
        let bool = builder.basic_type("bool", 1, Some(TypeEncoding::boolean), DIFlags::Private);

        Self {
            str,
            unit,
            bool,
            int_types: BinarySearchMap::new().into(),
            never,
        }
    }
}
