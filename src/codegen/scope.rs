use std::collections::HashMap;

use wllvm::value::FnValue;

use super::types::{Type, TypedValue};

pub struct Scope<'p, 'ctx> {
    parent: Option<&'p Scope<'p, 'ctx>>,
    variables: HashMap<String, TypedValue<'ctx>>,
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

    pub fn with_params(mut self, params: &[(&str, Type)], function: FnValue<'ctx>) -> Self {
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
            );
        }

        self
    }

    pub fn create_variable(&mut self, name: &str, val: TypedValue<'ctx>) {
        self.variables.insert(name.to_owned(), val);
    }

    pub fn get_variable(&self, name: &str) -> Option<&TypedValue<'ctx>> {
        self.variables
            .get(name)
            .or_else(|| self.parent?.get_variable(name))
    }
}
