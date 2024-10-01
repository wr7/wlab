use crate::{
    codegen::{
        self,
        codegen_context::{CodegenContext, Crate},
        namestore::{FieldInfo, NameStoreEntry},
        types::Type,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast,
    util,
};

impl<'ctx> CodegenContext<'ctx> {
    pub(super) fn generate_struct_bodies(
        &mut self,
        ast: &ast::Module,
        source: &str,
        crate_: &Crate<'ctx>,
    ) -> Result<(), Diagnostic> {
        for struct_ in &ast.structs {
            let mut packed = false;

            for attr in &struct_.attributes {
                match &**attr {
                    ast::Attribute::Packed => packed = true,
                    _ => return Err(codegen::error::non_struct_attribute(attr)),
                }
            }

            let line_no = util::line_and_col(source, struct_.1.start).0 as u32;

            let mut fields = Vec::new();
            let mut field_names: Vec<S<&str>> = Vec::new();

            for field in &struct_.fields {
                match field_names.binary_search_by(|f| f.cmp(field.name)) {
                    Ok(idx) => {
                        let field1 = field_names[idx];
                        return Err(codegen::error::duplicate_field(field1, field.1));
                    }
                    Err(idx) => field_names.insert(idx, S(field.name, field.1)),
                }

                let line_no = util::line_and_col(source, field.1.start).0 as u32;
                let ty = Type::new(self, &crate_.name, &field.type_)?;

                fields.push(FieldInfo {
                    name: field.name.to_owned(),
                    ty,
                    line_no,
                });
            }

            let llvm_fields: Option<Vec<wllvm::Type>> = fields
                .iter()
                .map(|field| field.ty.llvm_type(&self))
                .collect();

            let NameStoreEntry::Struct(struct_info) = self
                .name_store
                .get_item_in_crate_mut(&crate_.name, struct_.name)
            else {
                unreachable!()
            };

            if let Some(llvm_fields) = llvm_fields {
                struct_info
                    .llvm_type
                    .as_ref()
                    .unwrap()
                    .set_body(&llvm_fields, packed);
            } else {
                struct_info.llvm_type = None;
            }

            struct_info.fields = fields;
            struct_info.packed = packed;
            struct_info.line_no = line_no;
            struct_info.file_no = crate_.file_no;
        }

        Ok(())
    }
}
