use std::collections::HashMap;

use wllvm::value::FnValue;
use wutil::Span;

use crate::{
    codegen::{
        types::Type,
        values::{GenericValue, RValue},
    },
    error_handling::Spanned as S,
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
}

impl<'ctx> Scope<'_, 'ctx> {
    pub fn new_global() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
            break_context: None,
            return_type: None,
        }
    }
}

impl<'p, 'ctx> Scope<'p, 'ctx> {
    pub fn new(parent: &'p Scope<'_, 'ctx>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
            break_context: None,
            return_type: None,
        }
    }

    pub fn with_return_type(mut self, return_type: Type) -> Self {
        self.return_type = Some(return_type);
        self
    }

    pub fn with_params(mut self, params: &[(S<&str>, Type)], function: FnValue<'ctx>) -> Self {
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
            );
        }

        self
    }

    /// Creates a function scope for an uncallable function (ie one of its parameters is uninstantiable)
    pub fn with_uninstatiable_params(mut self, params: &[(S<&str>, Type)]) -> Self {
        for param in params.iter() {
            self.create_variable(
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

    pub fn create_variable(&mut self, name: S<&str>, value: GenericValue<'ctx>) {
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
