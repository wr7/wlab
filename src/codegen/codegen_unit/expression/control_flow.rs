use wllvm::value::ValueEnum;
use wutil::Span;

use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        error,
        scope::{BreakContext, Scope},
        types::Type,
        values::RValue,
        warning,
    },
    error_handling::{self, Diagnostic, Spanned as S},
    parser::ast::{self, Expression},
};

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_break(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        value: Option<S<&Expression>>,
        span: Span,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let vspan = value.map_or(span, |S(_, s)| s);

        let Some(break_context) = scope.get_break() else {
            return Err(error::break_outside_of_loop(span));
        };

        let value = value.map(|v| self.generate_rvalue(v, scope)).transpose()?;

        break_context.build_break(&self, value, vspan)?;

        let new_bb = self
            .c
            .context
            .insert_basic_block_after(self.builder.current_block().unwrap(), c"");
        self.builder.position_at_end(new_bb);

        Ok(RValue {
            val: None,
            type_: Type::never,
        })
    }

    pub fn generate_loop(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        block: S<&ast::CodeBlock>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let prev_block = self.builder.current_block().unwrap();

        let bb = self.c.context.insert_basic_block_after(prev_block, c"");
        let jump_to = self.c.context.insert_basic_block_after(bb, c"");
        let break_context = BreakContext::new(jump_to);

        self.builder.build_br(bb);
        self.builder.position_at_end(bb);

        let mut inner_scope = Scope::new(scope).with_break(&break_context);

        self.generate_codeblock(&block, &mut inner_scope)?;

        self.builder.build_br(bb);

        self.builder.position_at_end(jump_to);

        Ok(break_context.into_rvalue())
    }

    pub fn generate_if(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        condition: &S<ast::Expression>,
        block: S<&ast::CodeBlock>,
        else_block: &Option<S<ast::CodeBlock>>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let condition_span = condition.1;
        let condition = self.generate_rvalue(condition.as_sref(), scope)?;

        if !condition.type_.is(&Type::bool) {
            return Err(codegen::error::unexpected_type(
                condition_span,
                &Type::bool,
                &condition.type_,
            ));
        }

        let Some(base_bb) = self.builder.current_block() else {
            unreachable!()
        };

        let if_bb = self.c.context.insert_basic_block_after(base_bb, c"");
        let else_bb = else_block
            .as_ref()
            .map(|_| self.c.context.insert_basic_block_after(if_bb, c""));
        let continuing_bb = self.c.context.insert_basic_block_after(if_bb, c"");

        if let Some(condition_val) = condition.val {
            let Some(ValueEnum::IntValue(condition)) = condition_val.downcast() else {
                unreachable!()
            };

            self.builder
                .build_cond_br(condition, if_bb, else_bb.unwrap_or(continuing_bb));
        } else {
            let dead_span = if let Some(else_block) = else_block {
                error_handling::span_of(&[block, else_block.as_sref()])
            } else {
                error_handling::span_of(&[block])
            }
            .unwrap();

            self.c.warnings.push((
                self.file_no,
                warning::unreachable_code(condition_span, dead_span),
            ));

            self.builder.build_unreachable();
        };

        self.builder.position_at_end(if_bb);

        let mut if_scope = Scope::new(scope);
        let if_retval = self.generate_codeblock(*block, &mut if_scope)?;

        if if_retval.val.is_some() {
            self.builder.build_br(continuing_bb);
        } else {
            self.builder.build_unreachable();
        }

        let else_retval: Option<RValue<'ctx>> =
            if let Some((else_bb, else_block)) = else_bb.zip(else_block.as_ref()) {
                self.builder.position_at_end(else_bb);
                let mut else_scope = Scope::new(scope);

                let else_retval = self.generate_codeblock(else_block, &mut else_scope)?;

                if else_retval.val.is_some() {
                    self.builder.build_br(continuing_bb);
                } else {
                    self.builder.build_unreachable();
                }

                Some(else_retval)
            } else {
                None
            };

        self.builder.position_at_end(continuing_bb);

        let Some(else_retval) = else_retval else {
            return Ok(RValue {
                type_: Type::unit,
                val: Some(*self.c.core_types.unit.const_(&[])),
            });
        };

        if let Some((type_, val)) = if_retval
            .val
            .map(|v| (&if_retval.type_, v))
            .xor(else_retval.val.map(|v| (&else_retval.type_, v)))
        {
            return Ok(RValue {
                val: Some(val),
                type_: type_.clone(),
            });
        }

        let Some((if_val, else_val)) = if_retval.val.zip(else_retval.val) else {
            return Ok(RValue {
                val: None,
                type_: Type::never,
            });
        };

        if else_retval.type_ != if_retval.type_ {
            return Err(codegen::error::mismatched_if_else(
                S(&if_retval.type_, block.1),
                S(&else_retval.type_, else_block.as_ref().unwrap().1),
            ));
        }

        let phi = self.builder.build_phi(if_val.type_(), c"");

        phi.add_incoming(&[if_val, else_val], &[if_bb, else_bb.unwrap()]);

        Ok(RValue {
            type_: if_retval.type_,
            val: Some(*phi),
        })
    }
}
