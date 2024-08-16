use crate::{
    codegen::{
        self,
        namestore::NameStoreEntry,
        scope::Scope,
        types::{Type, TypedValue},
        CodegenUnit,
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Attribute, Visibility},
    util,
};

use wllvm::debug_info::{DIFlags, DIType};
use wutil::Span;

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_function(
        &mut self,
        function: &S<ast::Function>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<(), Diagnostic> {
        let Ok(NameStoreEntry::Function(function_info)) = self
            .c
            .name_store
            .get_item_in_crate(self.crate_name, S(function.name, Span::at(0)))
        else {
            unreachable!()
        };

        let ll_function = function_info.function;

        let params: Result<Vec<(&str, Type)>, _> = function
            .params
            .iter()
            .map(|(n, t)| Ok((*n, Type::new(self.c, t)?)))
            .collect();
        let params = params?;

        let return_type = function_info.signature.return_type.clone();

        let param_dwarf_types: Vec<DIType> = std::iter::once(&return_type)
            .chain(params.iter().map(|(_, ty)| ty))
            .map(|ty| ty.get_dwarf_type(self))
            .collect();

        let di_flags = if function.visibility == ast::Visibility::Public {
            DIFlags::Public
        } else {
            DIFlags::Private
        };

        let dwarf_subprogram = self.debug_context.builder.subroutine_type(
            self.debug_context.cu.file(),
            &param_dwarf_types,
            di_flags,
        );

        let (scope_line_no, scope_col_no) = util::line_and_col(self.source, function.body.1.start);
        let (fn_line_no, _) = util::line_and_col(self.source, function.1.start);

        let subprogram = self.debug_context.builder.subprogram(
            self.debug_context.scope,
            function.name,
            ll_function.name(),
            self.debug_context.cu.file(),
            fn_line_no as u32,
            scope_line_no as u32,
            dwarf_subprogram,
            function.visibility == Visibility::Private,
            true,
            true,
            di_flags,
        );

        ll_function.set_subprogram(subprogram);

        let dbg_lexical_block = self.debug_context.builder.lexical_block(
            **subprogram,
            self.debug_context.cu.file(),
            scope_line_no as u32,
            scope_col_no as u32,
        );

        let mut dbg_scope = **dbg_lexical_block;
        std::mem::swap(&mut dbg_scope, &mut self.debug_context.scope);

        let mut intrinsic_span = None;

        for attr in &function.attributes {
            if let Attribute::Intrinsic(intrinsic) = **attr {
                if let Some(first_intrinsic) = intrinsic_span {
                    return Err(codegen::error::multiple_intrinsic_attributes(
                        first_intrinsic,
                        attr.1,
                    ));
                }

                self.add_intrinsic(function, function_info, &params, S(intrinsic, attr.1))?;

                intrinsic_span = Some(attr.1);
            }
        }

        if intrinsic_span.is_some() {
            // there was an intrinsic attribute; skip body generation //
            std::mem::swap(&mut dbg_scope, &mut self.debug_context.scope);
            return Ok(());
        }

        let main_block = ll_function.add_basic_block(c"");
        self.builder.position_at_end(main_block);

        let mut fn_scope = Scope::new(scope).with_params(&params, ll_function);

        let return_value = self.generate_codeblock(&function.body, &mut fn_scope)?;

        if return_value.type_ != return_type {
            return Err(codegen::error::incorrect_return_type(
                function.body.as_sref(),
                &return_type,
                &return_value.type_,
            ));
        }

        self.builder.build_ret(return_value.val);

        std::mem::swap(&mut dbg_scope, &mut self.debug_context.scope);

        Ok(())
    }

    /// Generates a codeblock: NOTE: this will NOT create a new scope. The caller should create one for this block
    pub fn generate_codeblock(
        &self,
        block: &ast::CodeBlock,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let mut statements: &[S<ast::Statement>] = &block.body;
        let implicit_return: Option<S<&ast::Expression>>;

        if block.trailing_semicolon.is_none() {
            if let Some((last_statement, statements_)) = statements.split_last() {
                if let ast::Statement::Expression(expr) = &**last_statement {
                    implicit_return = Some(S(expr, last_statement.1));
                    statements = statements_;
                } else {
                    implicit_return = None;
                };
            } else {
                implicit_return = None;
            };
        } else {
            implicit_return = None;
        }

        for statement in statements {
            self.generate_statement(scope, statement.as_sref())?;
        }

        let return_value: TypedValue = implicit_return
            .map(|r| self.generate_expression(r, scope))
            .transpose()?
            .unwrap_or_else(|| TypedValue {
                type_: Type::unit,
                val: *self.c.core_types.unit.const_(&[]),
            });

        Ok(return_value)
    }
}
