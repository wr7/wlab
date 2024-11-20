use std::collections::HashMap;

use wllvm::{
    debug_info::{DIFlags, DILocalScope},
    value::FnValue,
};
use wutil::Span;

use crate::{
    codegen::{
        codegen_unit::CodegenUnit,
        types::Type,
        values::{GenericValue, MutValue, RValue},
    },
    error_handling::Spanned as S,
    util::{self, line_and_col},
};

mod break_;
pub use break_::BreakContext;

pub struct ScopeVariable<'ctx> {
    pub value: GenericValue<'ctx>,
    pub name_span: Span,
}

pub struct Scope<'p, 'ctx> {
    parent: Option<&'p Scope<'p, 'ctx>>,
    variables: HashMap<String, ScopeVariable<'ctx>>,
    break_context: Option<&'p BreakContext<'ctx>>,
    return_type: Option<Type>,
    di_scope: Option<DILocalScope<'ctx>>,
}

impl<'ctx> Scope<'_, 'ctx> {
    pub fn new_global() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
            break_context: None,
            return_type: None,
            di_scope: None,
        }
    }
}

impl<'p, 'ctx> Scope<'p, 'ctx> {
    pub fn new(cu: &CodegenUnit<'_, 'ctx>, parent: &'p Scope<'_, 'ctx>, start: usize) -> Self {
        let Some(parent_di_scope) = parent.di_scope else {
            unreachable!()
        };

        Scope::new_function(cu, parent, parent_di_scope, start)
    }

    // TODO: remove DebugContext::scope
    pub fn new_function(
        cu: &CodegenUnit<'_, 'ctx>,
        parent: &'p Scope<'_, 'ctx>,
        parent_di_scope: DILocalScope<'ctx>,
        start: usize,
    ) -> Self {
        let (line, col) = line_and_col(cu.source, start);

        let di_scope = *cu.debug_context.builder.lexical_block(
            *parent_di_scope,
            cu.debug_context.cu.file(),
            line as u32,
            col as u32,
        );

        Self {
            parent: Some(parent),
            variables: HashMap::new(),
            break_context: None,
            return_type: None,
            di_scope: Some(di_scope),
        }
    }

    pub fn with_return_type(mut self, return_type: Type) -> Self {
        self.return_type = Some(return_type);
        self
    }

    pub fn with_params(
        mut self,
        params: &[(S<&str>, Type)],
        function: FnValue<'ctx>,
        cu: &CodegenUnit<'_, 'ctx>,
    ) -> Self {
        for (i, param) in params.iter().enumerate() {
            let Some(val) = function.param(i as u32) else {
                unreachable!();
            };

            self.create_variable(
                param.0,
                GenericValue::RValue(RValue {
                    val: Some(val),
                    type_: param.1.clone(),
                }),
                cu,
                None,
            );
        }

        self
    }

    /// Creates a function scope for an uncallable function (ie one of its parameters is uninstantiable)
    pub fn with_uninstatiable_params(mut self, params: &[(S<&str>, Type)]) -> Self {
        for param in params.iter() {
            self.create_unreachable_variable(
                param.0,
                GenericValue::RValue(RValue {
                    val: None,
                    type_: param.1.clone(),
                }),
            );
        }

        self
    }

    pub fn with_break(mut self, break_context: &'p BreakContext<'ctx>) -> Self {
        self.break_context = Some(break_context);
        self
    }

    pub fn di_scope(&self) -> DILocalScope<'ctx> {
        self.di_scope.unwrap()
    }

    pub fn create_variable(
        &mut self,
        name: S<&str>,
        value: GenericValue<'ctx>,
        cu: &CodegenUnit<'_, 'ctx>,
        value_span: Option<Span>,
    ) {
        let (line, col) = util::line_and_col(cu.source, name.1.start);

        let ty = value.type_().llvm_type(cu.c);
        let dwarf_type = value.type_().get_dwarf_type(cu);

        let di_variable = ty.map(|ty| {
            cu.debug_context.builder.local_variable(
                self.di_scope(),
                *name,
                cu.debug_context.cu.file(),
                line as u32,
                dwarf_type,
                false,
                DIFlags::Zero,
                ty.alignment(&cu.c.target_data) * 8,
            )
        });

        let di_expr = cu.debug_context.builder.expression(&[]);

        let (e_line, e_col) =
            value_span.map_or((line, col), |s| util::line_and_col(cu.source, s.start));

        let di_loc =
            cu.c.context
                .debug_location(e_line as u32, e_col as u32, *self.di_scope(), None);

        match &value {
            GenericValue::RValue(RValue {
                val: Some(val),
                type_: _,
            }) => {
                cu.debug_context.builder.insert_dbg_value_at_end(
                    *val,
                    di_variable.unwrap(),
                    di_expr,
                    di_loc,
                    cu.builder.current_block().unwrap(),
                );
            }
            GenericValue::MutValue(MutValue {
                ptr: Some(ptr),
                type_: _,
            }) => {
                cu.debug_context.builder.insert_dbg_declare_at_end(
                    **ptr,
                    di_variable.unwrap(),
                    di_expr,
                    di_loc,
                    cu.builder.current_block().unwrap(),
                );
            }
            _ => {}
        }

        self.variables.insert(
            name.0.to_owned(),
            ScopeVariable {
                value,
                name_span: name.1,
            },
        );
    }

    fn create_unreachable_variable(&mut self, name: S<&str>, value: GenericValue<'ctx>) {
        self.variables.insert(
            name.0.to_owned(),
            ScopeVariable {
                value,
                name_span: name.1,
            },
        );
    }

    pub fn get_variable(&self, name: &str) -> Option<&ScopeVariable<'ctx>> {
        self.variables
            .get(name)
            .or_else(|| self.parent?.get_variable(name))
    }

    pub fn get_break(&self) -> Option<&'p BreakContext<'ctx>> {
        self.break_context.or_else(|| self.parent?.get_break())
    }

    pub fn get_return_type(&self) -> Option<&Type> {
        self.return_type
            .as_ref()
            .or_else(|| self.parent?.get_return_type())
    }
}
