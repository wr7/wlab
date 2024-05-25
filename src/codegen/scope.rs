use std::collections::HashMap;

use inkwell::values::{BasicValueEnum, FunctionValue, IntValue};

#[derive(Clone)]
pub struct FunctionInfo<'ctx> {
    pub num_params: usize,
    pub function: FunctionValue<'ctx>,
}

pub struct Scope<'p, 'ctx> {
    parent: Option<&'p Scope<'p, 'ctx>>,
    variables: HashMap<String, IntValue<'ctx>>,
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

    pub fn with_params<'a>(
        mut self,
        params: &'a [&'a str],
        function: &FunctionValue<'ctx>,
    ) -> Self {
        for (i, param) in params.into_iter().enumerate() {
            let Some(BasicValueEnum::IntValue(val)) = function.get_nth_param(i as u32) else {
                unreachable!();
            };

            self.create_variable(param, val);
        }

        self
    }

    pub fn create_variable(&mut self, name: &str, val: IntValue<'ctx>) {
        self.variables.insert(name.to_owned(), val);
    }

    pub fn create_function(&mut self, name: &'_ str, function: FunctionInfo<'ctx>) {
        self.functions.insert(name.to_owned(), function);
    }

    pub fn get_variable<'a>(&'a self, name: &'_ str) -> Option<&'a IntValue<'ctx>> {
        self.variables.get(name)
    }

    pub fn get_function<'a>(&'a self, name: &'_ str) -> Option<&'a FunctionInfo<'ctx>> {
        self.functions
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.get_function(name)))
    }
}
