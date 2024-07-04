use std::collections::HashMap;

use inkwell::values::FunctionValue;

use super::types::{Type, TypedValue};

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub return_type: Type,
}

#[derive(Clone, Debug)]
pub struct FunctionInfo<'ctx> {
    pub signature: FunctionSignature,
    pub function: FunctionValue<'ctx>,
}

pub struct Scope<'p, 'ctx> {
    parent: Option<&'p Scope<'p, 'ctx>>,
    variables: HashMap<String, TypedValue<'ctx>>,
    functions: HashMap<String, FunctionInfo<'ctx>>,
}

impl<'ctx> Scope<'static, 'ctx> {
    pub fn new_global() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

impl<'p, 'ctx> Scope<'p, 'ctx> {
    pub fn new(parent: &'p Scope<'_, 'ctx>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn with_params(mut self, params: &[(&str, Type)], function: FunctionValue<'ctx>) -> Self {
        for (i, param) in params.iter().enumerate() {
            let Some(val) = function.get_nth_param(i as u32) else {
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

    pub fn create_function(&mut self, name: &str, function: FunctionInfo<'ctx>) {
        self.functions.insert(name.to_owned(), function);
    }

    pub fn get_variable(&self, name: &str) -> Option<&TypedValue<'ctx>> {
        self.variables.get(name)
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo<'ctx>> {
        self.functions
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.get_function(name)))
    }
}
