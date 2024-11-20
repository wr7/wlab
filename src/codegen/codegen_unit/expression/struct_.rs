use wllvm::{type_::StructType, value::StructValue};

use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        error,
        scope::Scope,
        types::Type,
        values::{MutValue, RValue},
        warning,
    },
    error_handling::{self, Diagnostic, Spanned as S},
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
            .unwrap()
            .as_struct()
            .unwrap();

        let idx = struct_info
            .fields
            .iter()
            .position(|fi| fi.name == **field)
            .ok_or_else(|| codegen::error::invalid_field(&path, *field))?;

        let Some(lhs_val) = lhs.val else {
            return Ok(RValue {
                val: None,
                type_: struct_info.fields[idx].ty.clone(),
            });
        };

        let Ok(lhs) = StructValue::try_from(lhs_val) else {
            unreachable!()
        };

        let val = self
            .builder
            .build_extract_value(lhs, idx as u32, c"")
            .unwrap();

        Ok(RValue {
            val: Some(val),
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
            .unwrap()
            .as_struct()
            .unwrap();
        let idx = struct_info
            .fields
            .iter()
            .position(|f| f.name == **field)
            .ok_or_else(|| error::invalid_field(path, *field))?;

        let Some((lhs_ptr, lhs_llvm_type)) = lhs.ptr.zip(lhs.type_.llvm_type(&self.c)) else {
            return Ok(MutValue {
                ptr: None,
                type_: struct_info.fields[idx].ty.clone(),
            });
        };

        let isize = self.c.core_types.isize;
        let field_ptr = self.builder.build_gep(
            lhs_llvm_type,
            lhs_ptr,
            &[isize.const_(0, false), isize.const_(idx as u64, false)],
            c"",
        );

        Ok(MutValue {
            ptr: Some(field_ptr),
            type_: struct_info.fields[idx].ty.clone(),
        })
    }

    pub(crate) fn generate_struct(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        name: &S<util::MaybeVec<S<&str>>>,
        fields: &Vec<S<ast::StructInitializerField>>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        struct AssignedField<'a, 'ctx> {
            src_idx: usize,
            name: S<&'a str>,
            value: S<RValue<'ctx>>,
        }

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
            .unwrap()
            .as_struct()
            .unwrap();

        let mut assigned_fields = Vec::<S<AssignedField>>::new();

        for (src_idx, field) in fields.iter().enumerate() {
            let idx = match assigned_fields.binary_search_by(|f| f.name.cmp(&field.name)) {
                Ok(idx) => {
                    let first = &assigned_fields[idx];

                    return Err(error::duplicate_field(first.name, field.name.1));
                }
                Err(idx) => idx,
            };

            let mut scope = Scope::new(self, scope, field.1.start);
            let value = S(
                self.generate_rvalue(field.val.as_sref(), &mut scope)?,
                field.val.1,
            );

            assigned_fields.insert(
                idx,
                S(
                    AssignedField {
                        src_idx,
                        name: field.name,
                        value,
                    },
                    field.1,
                ),
            );
        }

        let mut field_values = Vec::<wllvm::Value>::new();
        let mut first_diverging_src_idx: Option<usize> = None;

        for field in &struct_info.fields {
            let assigned_val = match assigned_fields.binary_search_by(|v| v.name.cmp(&field.name)) {
                Ok(idx) => &assigned_fields[idx],
                Err(_) => return Err(error::missing_field(&field.name, S(path, name.1))),
            };

            let val = &assigned_val.value;
            if !val.type_.is(&field.ty) {
                return Err(error::unexpected_type(val.1, &field.ty, &val.type_));
            }

            let Some(val) = val.val else {
                first_diverging_src_idx = Some(
                    first_diverging_src_idx
                        .map_or(assigned_val.src_idx, |i| i.min(assigned_val.src_idx)),
                );

                continue;
            };

            field_values.push(val);
        }

        if struct_info.fields.len() != assigned_fields.len() {
            for field in assigned_fields {
                if !struct_info.fields.iter().any(|s| s.name == *field.name) {
                    return Err(error::invalid_field(path, field.name));
                }
            }
        }

        if let Some(idx) = first_diverging_src_idx {
            if let Some(dead_code) = error_handling::span_of(&fields[idx + 1..]) {
                self.c.warnings.push((
                    self.file_no,
                    warning::unreachable_code(fields[idx].val.1, dead_code),
                ))
            }

            return Ok(RValue { val: None, type_ });
        }

        let llvm_type = StructType::try_from(type_.llvm_type(self.c).unwrap()).unwrap();

        let val = Some(*llvm_type.const_(&field_values));

        Ok(RValue { val, type_ })
    }
}
