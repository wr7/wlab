use crate::{
    codegen::{
        self,
        codegen_unit::CodegenUnit,
        error,
        scope::Scope,
        types::Type,
        values::{GenericValue, MutValue, RValue},
        warning,
    },
    error_handling::{self, Diagnostic, Spanned as S},
    parser::ast::{self, Expression, Literal, Path, Visibility},
    util,
};

use wllvm::value::Linkage;
use wutil::Span;

mod control_flow;
mod struct_;

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_rvalue(
        &self,
        expression: S<&ast::Expression>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<RValue<'ctx>, Diagnostic> {
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
                "true" => Ok(RValue {
                    val: Some(*self.c.core_types.bool.const_(1, false)),
                    type_: Type::bool,
                }),
                "false" => Ok(RValue {
                    val: Some(*self.c.core_types.bool.const_(0, false)),
                    type_: Type::bool,
                }),
                _ => {
                    let Some(var) = scope.get_variable(ident) else {
                        return Err(codegen::error::undefined_variable(S(ident, expression.1)));
                    };

                    Ok(var.value.clone().into_rvalue(self))
                }
            },
            Expression::Literal(lit) => self.generate_literal(S(lit, expression.1)),
            Expression::BinaryOperator(a_expr, operator, b_expr) => {
                let a = self.generate_rvalue(a_expr.as_sref(), scope)?;
                let b = self.generate_rvalue(b_expr.as_sref(), scope)?;

                a.generate_operation(&self, a_expr.1, *operator, &S(b, b_expr.1))
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
            Expression::Loop(block) => self.generate_loop(scope, block.as_sref()),
            Expression::StructInitializer { name, fields } => {
                self.generate_struct(scope, name, fields)
            }
            Expression::FieldAccess(lhs, field) => self.generate_field_access(scope, lhs, field),
            Expression::Break(value) => {
                self.generate_break(scope, value.as_ref().map(|v| v.as_sref()), expression.1)
            }
            Expression::Return(_val) => unimplemented!(),
        }
    }

    pub fn generate_mutvalue(
        &self,
        expression: S<&ast::Expression>,
        scope: &mut Scope<'_, 'ctx>,
    ) -> Result<MutValue<'ctx>, Diagnostic> {
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
                "true" | "false" => None,
                _ => {
                    let Some(var) = scope.get_variable(ident) else {
                        return Err(codegen::error::undefined_variable(S(ident, expression.1)));
                    };

                    let GenericValue::MutValue(mval) = &var.value else {
                        return Err(error::modified_immutable_variable(
                            S(ident, var.name_span),
                            expression.1,
                        ));
                    };

                    Some(Ok(mval.clone()))
                }
            },
            Expression::FieldAccess(lhs, field) => {
                Some(self.generate_mutable_field_access(scope, lhs, field))
            }
            _ => None,
        }
        .ok_or_else(|| error::modify_rvalue(expression.1))?
    }

    fn generate_literal(&self, literal: S<&ast::Literal>) -> Result<RValue<'ctx>, Diagnostic> {
        match *literal {
            Literal::Number(num) => self.generate_number_literal(num, literal.1),
            Literal::String(str) => Ok(self.generate_string_literal(str)),
        }
    }

    fn generate_string_literal(&self, lit: &str) -> RValue<'ctx> {
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

        RValue {
            val: Some(*self.c.core_types.str.const_(&[*string_ptr, *str_len])),
            type_: Type::str,
        }
    }

    fn generate_number_literal(&self, lit: &str, span: Span) -> Result<RValue<'ctx>, Diagnostic> {
        let idx = lit.find(|c: char| !c.is_ascii_digit()).unwrap_or(lit.len());
        let suffix = &lit[idx..];

        let type_ = if suffix.is_empty() {
            Type::i(32)
        } else {
            Type::i(
                suffix
                    .strip_prefix("i")
                    .and_then(|s| s.parse::<u32>().ok())
                    .ok_or_else(|| codegen::error::invalid_number(S(lit, span)))?,
            )
        };

        Ok(RValue {
            val: Some(
                *self
                    .c
                    .context
                    .int_type(32)
                    .const_from_string(&lit[..idx], 10)
                    .ok_or_else(|| codegen::error::invalid_number(S(lit, span)))?,
            ),
            type_,
        })
    }

    fn generate_function_call(
        &self,
        span: Span,
        scope: &mut Scope<'_, 'ctx>,
        fn_name: &S<Path>,
        arguments: &[S<ast::Expression>],
    ) -> Result<RValue<'ctx>, Diagnostic> {
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

        let fn_name = function.function.name();

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
        let mut uncallable = false;

        for (i, arg) in arguments.iter().enumerate() {
            let arg_span = arg.1;
            let arg = self.generate_rvalue(arg.as_sref(), scope)?;

            let expected_type = &signature.params[i];
            if !arg.type_.is(expected_type) {
                return Err(codegen::error::unexpected_type(
                    arguments[i].1,
                    expected_type,
                    &arg.type_,
                ));
            }

            let Some(arg_val) = arg.val else {
                uncallable = true;

                let Some(dead_code) = error_handling::span_of(&arguments[i + 1..]) else {
                    continue;
                };

                self.c
                    .warnings
                    .push((self.file_no, warning::unreachable_code(arg_span, dead_code)));
                continue;
            };

            metadata_arguments.push(arg_val);
        }

        if uncallable {
            return Ok(RValue {
                val: None,
                type_: function.signature.return_type,
            });
        }

        let ret_val = self
            .builder
            .build_fn_call(mod_function, &metadata_arguments, c"");

        if function.signature.return_type.llvm_type(self.c).is_none() {
            return Ok(RValue {
                val: None,
                type_: function.signature.return_type,
            });
        }

        Ok(RValue {
            val: Some(ret_val),
            type_: function.signature.return_type,
        })
    }
}
