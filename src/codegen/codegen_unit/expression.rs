use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        scope::Scope,
        types::{Type, TypedValue},
    },
    error_handling::{Diagnostic, Spanned as S},
    parser::ast::{self, Expression, Literal, Path, Visibility},
    util,
};

use wllvm::value::{Linkage, ValueEnum};
use wutil::Span;

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_expression(
        &self,
        expression: S<&ast::Expression>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let (line_no, col_no) = util::line_and_col(self.source, expression.1.start);

        let dbg_location = self.c.context.debug_location(
            line_no as u32,
            col_no as u32,
            self.debug_context.scope,
            None,
        );

        self.builder.set_debug_location(dbg_location);

        match *expression {
            Expression::Identifier(ident) => match *ident {
                "true" => Ok(TypedValue {
                    val: self.c.core_types.bool.const_(1, false).into(),
                    type_: Type::bool,
                }),
                "false" => Ok(TypedValue {
                    val: self.c.core_types.bool.const_(0, false).into(),
                    type_: Type::bool,
                }),
                _ => scope
                    .get_variable(ident)
                    .cloned()
                    .ok_or(codegen::error::undefined_variable(S(ident, expression.1))),
            },
            Expression::Literal(lit) => self.generate_literal(S(lit, expression.1)),
            Expression::BinaryOperator(a_expr, operator, b_expr) => {
                let a = self.generate_expression(a_expr.as_sref(), scope)?;
                let b = self.generate_expression(b_expr.as_sref(), scope)?;

                a.generate_operation(&self.builder, a_expr.1, *operator, &S(b, b_expr.1))
            }
            Expression::CompoundExpression(block) => {
                let mut scope = Scope::new(scope);
                self.generate_codeblock(block, &mut scope)
            }
            Expression::FunctionCall(fn_name, arguments) => {
                self.generate_function_call(expression.1, scope, fn_name, arguments)
            }
            Expression::If {
                condition,
                block,
                else_block,
            } => self.generate_if(scope, condition, block.as_sref(), else_block),
        }
    }

    fn generate_if(
        &self,
        scope: &mut Scope<'_, 'ctx>,
        condition: &S<ast::Expression>,
        block: S<&ast::CodeBlock>,
        else_block: &Option<S<ast::CodeBlock>>,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let condition_span = condition.1;
        let condition = self.generate_expression(condition.as_sref(), scope)?;

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
        let else_retval: Option<TypedValue<'ctx>> = if let Some(else_block) = else_block {
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

            TypedValue {
                type_: if_retval.type_,
                val: *phi,
            }
        } else {
            TypedValue {
                type_: Type::unit,
                val: *self.c.core_types.unit.const_null(),
            }
        };

        Ok(retval)
    }

    fn generate_literal(&self, literal: S<&ast::Literal>) -> Result<TypedValue<'ctx>, Diagnostic> {
        match *literal {
            Literal::Number(num) => self.generate_number_literal(num, literal.1),
            Literal::String(str) => Ok(self.generate_string_literal(str)),
        }
    }

    fn generate_string_literal(&self, lit: &str) -> TypedValue<'ctx> {
        let string = self.c.context.const_string(lit, false);

        let string_global = self.module.add_global(
            *self.c.context.int_type(8).array_type(lit.len() as u64),
            c"",
        );

        string_global.set_initializer(Some(*string));
        string_global.set_constant(true);
        string_global.set_linkage(Linkage::Private);

        let string_ptr = string_global.as_ptr();
        let str_len = self.c.core_types.isize.const_(lit.len() as u64, false);

        TypedValue {
            val: *self.c.core_types.str.const_(&[*string_ptr, *str_len]),
            type_: Type::str,
        }
    }

    fn generate_number_literal(
        &self,
        lit: &str,
        span: Span,
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        Ok(TypedValue {
            val: *self
                .c
                .context
                .int_type(32)
                .const_from_string(lit, 10)
                .ok_or(codegen::error::invalid_number(S(lit, span)))?,
            type_: Type::i32,
        })
    }

    fn generate_function_call(
        &self,
        span: Span,
        scope: &mut Scope<'_, 'ctx>,
        fn_name: &S<Path>,
        arguments: &[S<ast::Expression>],
    ) -> Result<TypedValue<'ctx>, Diagnostic> {
        let function = if let [fn_name] = &***fn_name {
            self.c
                .name_store
                .get_item_in_crate(self.crate_name, *fn_name)?
                .as_function()
                .ok_or_else(|| codegen::error::not_function(*fn_name))?
                .clone()
        } else if let [parent_crate, fn_direct_name] = &***fn_name {
            let func = self
                .c
                .name_store
                .get_item(fn_name)?
                .as_function()
                .ok_or_else(|| codegen::error::not_function_path(fn_name))?
                .clone();

            if func.visibility != Visibility::Public && **parent_crate != self.crate_name {
                return Err(codegen::error::private_function(
                    *parent_crate,
                    *fn_direct_name,
                ));
            }

            func
        } else {
            todo!("modules are not implemented yet")
        };

        let fn_name = &function.name;

        // Add a declaration to this module if it doesn't already exist //
        let mod_function = self.module.get_function(fn_name).unwrap_or_else(|| {
            let func = self.module.add_function(c"", function.function.type_());
            func.set_name(fn_name);
            func.set_linkage(Linkage::External);
            func
        });

        let signature = &function.signature;

        if arguments.len() != signature.params.len() {
            return Err(codegen::error::invalid_param_count(
                span,
                signature.params.len(),
                arguments.len(),
            ));
        }

        let mut metadata_arguments: Vec<wllvm::Value> = Vec::with_capacity(arguments.len());

        for (i, arg) in arguments.iter().enumerate() {
            let arg = self.generate_expression(arg.as_sref(), scope)?;

            metadata_arguments.push(arg.val);

            let expected_type = &signature.params[i];
            if expected_type != &arg.type_ {
                return Err(codegen::error::unexpected_type(
                    arguments[i].1,
                    expected_type,
                    &arg.type_,
                ));
            }
        }

        let ret_val = self
            .builder
            .build_fn_call(mod_function, &metadata_arguments, c"");

        Ok(TypedValue {
            val: ret_val,
            type_: function.signature.return_type,
        })
    }
}
