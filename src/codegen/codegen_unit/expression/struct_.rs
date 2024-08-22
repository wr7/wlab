use wllvm::{type_::StructType, value::StructValue};

use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        error,
        scope::Scope,
        types::Type,
        values::{MutValue, RValue},
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Expression},
    util,
};

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub(crate) fn generate_field_access(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        lhs: &S<Expression>,
        field: &S<&str>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let lhs_span = lhs.1;
        let lhs = self.generate_rvalue(lhs.as_sref(), scope)?;

        let Type::Struct { path } = lhs.type_ else {
            return Err(codegen::error::non_struct_element_access(
                lhs_span, &lhs.type_, field,
            ));
        };

        let struct_info = self
            .c
            .name_store
            .get_item_from_string(&path)
            .as_struct()
            .unwrap();

        let idx = struct_info
            .fields
            .iter()
            .position(|fi| fi.name == **field)
            .ok_or_else(|| codegen::error::invalid_field(&path, *field))?;

        let Ok(lhs) = StructValue::try_from(lhs.val) else {
            unreachable!()
        };

        let val = self
            .builder
            .build_extract_value(lhs, idx as u32, c"")
            .unwrap();

        Ok(RValue {
            val,
            type_: struct_info.fields[idx].ty.clone(),
        })
    }

    pub fn generate_mutable_field_access(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        lhs: &S<Expression>,
        field: &S<&str>,
    ) -> Result<MutValue<'ctx>, Diagnostic> {
        let lhs_span = lhs.1;
        let lhs = self.generate_mutvalue(lhs.as_sref(), scope)?;
        let Type::Struct { path } = &lhs.type_ else {
            return Err(error::non_struct_element_access(
                lhs_span, &lhs.type_, field,
            ));
        };
        let struct_info = self
            .c
            .name_store
            .get_item_from_string(path)
            .as_struct()
            .unwrap();
        let idx = struct_info
            .fields
            .iter()
            .position(|f| f.name == **field)
            .ok_or_else(|| error::invalid_field(path, *field))?;
        let isize = self.c.core_types.isize;
        let field_ptr = self.builder.build_gep(
            lhs.type_.llvm_type(self.c),
            lhs.ptr,
            &[isize.const_(0, false), isize.const_(idx as u64, false)],
            c"",
        );
        Ok(MutValue {
            ptr: field_ptr,
            type_: struct_info.fields[idx].ty.clone(),
        })
    }

    pub(crate) fn generate_struct(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        name: &S<util::MaybeVec<S<&str>>>,
        fields: &Vec<S<ast::StructInitializerField>>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let type_ = Type::new(self.c, self.crate_name, name)?;

        let Type::Struct { path } = &type_ else {
            return Err(codegen::error::non_struct_type_initializer(S(
                &type_, name.1,
            )));
        };

        let struct_info = self
            .c
            .name_store
            .get_item_from_string(path)
            .as_struct()
            .unwrap();

        let mut assigned_fields = Vec::<S<(S<&str>, S<RValue<'ctx>>)>>::new();

        for field in fields {
            let idx = match assigned_fields.binary_search_by(|S(f, _)| f.0.cmp(&field.name)) {
                Ok(idx) => {
                    let first = &assigned_fields[idx];

                    return Err(error::duplicate_field(first.0 .0, field.name.1));
                }
                Err(idx) => idx,
            };

            let mut scope = Scope::new(scope);
            let val = S(
                self.generate_rvalue(field.val.as_sref(), &mut scope)?,
                field.val.1,
            );

            assigned_fields.insert(idx, S((field.name, val), field.1));
        }

        let mut field_values = Vec::<wllvm::Value>::new();

        for field in &struct_info.fields {
            let val = match assigned_fields.binary_search_by(|v| v.0 .0.cmp(&field.name)) {
                Ok(idx) => &assigned_fields[idx].0 .1,
                Err(_) => return Err(error::missing_field(&field.name, S(path, name.1))),
            };

            if val.type_ != field.ty {
                return Err(error::unexpected_type(val.1, &field.ty, &val.type_));
            }

            field_values.push(val.val);
        }

        if struct_info.fields.len() != assigned_fields.len() {
            for field in assigned_fields {
                if !struct_info.fields.iter().any(|s| s.name == *field.0 .0) {
                    return Err(error::invalid_field(path, field.0 .0));
                }
            }
        }

        let llvm_type = StructType::try_from(type_.llvm_type(self.c)).unwrap();

        let val = *llvm_type.const_(&field_values);

        Ok(RValue { val, type_ })
    }
}
