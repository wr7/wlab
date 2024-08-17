use std::collections::HashMap;

use wllvm::value::FnValue;
use wutil::Span;

use crate::{
    codegen::types::{Type, TypedValue},
    error_handling::Spanned as S,
};

pub struct ScopeVariable<'ctx> {
    /// NOTE: if variable is mutable, `value.val` is actually a mutable pointer of `value.type_`
    pub value: TypedValue<'ctx>,
    pub name_span: Span,
    pub mutable: bool,
}

pub struct Scope<'p, 'ctx> {
    parent: Option<&'p Scope<'p, 'ctx>>,
    variables: HashMap<String, ScopeVariable<'ctx>>,
}

impl<'ctx> Scope<'static, 'ctx> {
    pub fn new_global() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }
}

impl<'p, 'ctx> Scope<'p, 'ctx> {
    pub fn new(parent: &'p Scope<'_, 'ctx>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }

    pub fn with_params(mut self, params: &[(S<&str>, Type)], function: FnValue<'ctx>) -> Self {
        for (i, param) in params.iter().enumerate() {
            let Some(val) = function.param(i as u32) else {
                unreachable!();
            };

            self.create_variable(
                param.0,
                TypedValue {
                    val,
                    type_: param.1.clone(),
                },
                false,
            );
        }

        self
    }

    pub fn create_variable(&mut self, name: S<&str>, value: TypedValue<'ctx>, mutable: bool) {
        self.variables.insert(
            name.0.to_owned(),
            ScopeVariable {
                value,
                name_span: name.1,
                mutable,
            },
        );
    }

    pub fn get_variable(&self, name: &str) -> Option<&ScopeVariable<'ctx>> {
        self.variables
            .get(name)
            .or_else(|| self.parent?.get_variable(name))
    }
}
