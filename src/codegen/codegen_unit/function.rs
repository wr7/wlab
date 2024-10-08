use crate::{
    codegen::{
        self, namestore::NameStoreEntry, scope::Scope, types::Type, values::RValue, warning,
        CodegenUnit,
    },
    error_handling::{self, Diagnostic, Spanned as S},
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

        let params: Result<Vec<(S<&str>, Type)>, _> = function
            .params
            .iter()
            .map(|(n, t)| Ok((*n, Type::new(self.c, self.crate_name, t)?)))
            .collect();
        let params = params?;

        let uncallable = params.iter().any(|(_, ty)| ty.llvm_type(self.c).is_none());

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

        self.builder
            .set_debug_location(self.c.context.debug_location(
                scope_line_no as u32,
                scope_col_no as u32,
                self.debug_context.scope,
                None,
            ));

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

        let mut fn_scope = if uncallable {
            Scope::new(scope).with_uninstatiable_params(&params)
        } else {
            Scope::new(scope).with_params(&params, ll_function)
        }
        .with_return_type(return_type.clone());

        let return_value = self.generate_codeblock(&function.body, &mut fn_scope)?;

        if !return_value.type_.is(&return_type) {
            return Err(codegen::error::incorrect_implicit_return_type(
                function.body.as_sref(),
                &return_type,
                &return_value.type_,
            ));
        }

        if let Some(val) = return_value.val {
            self.builder.build_ret(val);
        } else {
            self.builder.build_unreachable();
        }

        std::mem::swap(&mut dbg_scope, &mut self.debug_context.scope);

        Ok(())
    }

    /// Generates a codeblock: NOTE: this will NOT create a new scope. The caller should create one for this block
    pub fn generate_codeblock(
        &self,
        block: &ast::CodeBlock,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let statements = &block.body;

        let implicit_return: Option<S<&ast::Expression>>;
        let mut other_statements: &[S<ast::Statement>] = &statements;

        if block.trailing_semicolon.is_none() {
            if let Some((last_statement, other_statements_)) = statements.split_last() {
                if let ast::Statement::Expression(expr) = &**last_statement {
                    implicit_return = Some(S(expr, last_statement.1));
                    other_statements = other_statements_;
                } else {
                    implicit_return = None;
                };
            } else {
                implicit_return = None;
            };
        } else {
            implicit_return = None;
        }

        // Index of first statement that yields the `!` type
        let mut terminating_idx = None;

        for (i, statement) in other_statements.iter().enumerate() {
            let retval = self.generate_statement(scope, statement.as_sref())?;

            if retval.is_some_and(|r| r.val.is_none()) {
                terminating_idx = terminating_idx.or(Some(i));
            }
        }

        let return_value: Option<RValue> = implicit_return
            .map(|r| self.generate_rvalue(r, scope))
            .transpose()?;

        if let Some(terminating_idx) = terminating_idx {
            if let Some(dead_code) = statements
                .get(terminating_idx + 1..)
                .and_then(error_handling::span_of)
            {
                let terminating_statement = statements[terminating_idx].1;

                self.c.warnings.push((
                    self.file_no,
                    warning::unreachable_code(terminating_statement, dead_code),
                ));
            }

            return Ok(RValue {
                val: None,
                type_: return_value.map_or(Type::never, |r| r.type_),
            });
        }

        Ok(return_value.unwrap_or_else(|| RValue {
            val: Some(*self.c.context.const_struct(&[], false)),
            type_: Type::unit,
        }))
    }
}
