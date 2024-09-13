use wllvm::value::ValueEnum;

use crate::{
    codegen::{self, codegen_unit::CodegenUnit, scope::Scope, types::Type, values::RValue},
    error_handling::{Diagnostic, Spanned as S},
    parser::ast,
};

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_loop(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        block: S<&ast::CodeBlock>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
        let prev_block = self.builder.current_block().unwrap();

        let bb = self.c.context.insert_basic_block_after(prev_block, c"");
        self.builder.build_br(bb);

        self.builder.position_at_end(bb);

        let mut inner_scope = Scope::new(scope);

        self.generate_codeblock(&block, &mut inner_scope)?;

        self.builder.build_br(bb);

        let new_block = self
            .c
            .context
            .insert_basic_block_after(self.builder.current_block().unwrap(), c"");

        self.builder.position_at_end(new_block);

        let unit = self.c.context.const_struct(&[], false);

        Ok(RValue {
            val: *unit,
            type_: Type::unit,
        })
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

        if condition.type_ != Type::bool {
            return Err(codegen::error::unexpected_type(
                condition_span,
                &Type::bool,
                &condition.type_,
            ));
        }

        let Some(ValueEnum::IntValue(condition)) = condition.val.downcast() else {
            unreachable!()
        };

        let Some(base_bb) = self.builder.current_block() else {
            unreachable!()
        };

        let if_bb = self.c.context.insert_basic_block_after(base_bb, c"");
        let continuing_bb = self.c.context.insert_basic_block_after(if_bb, c"");

        self.builder.position_at_end(if_bb);

        let mut if_scope = Scope::new(scope);
        let if_retval = self.generate_codeblock(*block, &mut if_scope)?;

        self.builder.build_br(continuing_bb);

        let else_bb;
        let else_retval: Option<RValue<'ctx>> = if let Some(else_block) = else_block {
            let else_bb_ = self.c.context.insert_basic_block_after(if_bb, c"");
            else_bb = Some(else_bb_);

            self.builder.position_at_end(else_bb_);
            let mut else_scope = Scope::new(scope);

            let else_retval = self.generate_codeblock(else_block, &mut else_scope)?;

            self.builder.build_br(continuing_bb);

            Some(else_retval)
        } else {
            else_bb = None;
            None
        };

        self.builder.position_at_end(base_bb);
        self.builder
            .build_cond_br(condition, if_bb, else_bb.unwrap_or(continuing_bb));

        self.builder.position_at_end(continuing_bb);

        let retval = if let Some(else_retval) = else_retval {
            if else_retval.type_ != if_retval.type_ {
                return Err(codegen::error::mismatched_if_else(
                    S(&if_retval.type_, block.1),
                    S(&else_retval.type_, else_block.as_ref().unwrap().1),
                ));
            }

            let phi = self.builder.build_phi(if_retval.val.type_(), c"");

            phi.add_incoming(
                &[if_retval.val, else_retval.val],
                &[if_bb, else_bb.unwrap()],
            );

            RValue {
                type_: if_retval.type_,
                val: *phi,
            }
        } else {
            RValue {
                type_: Type::unit,
                val: *self.c.core_types.unit.const_(&[]),
            }
        };

        Ok(retval)
    }
}
