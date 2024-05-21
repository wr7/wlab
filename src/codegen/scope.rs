use std::{cell::RefCell, collections::HashMap, marker::PhantomData, mem, ptr::NonNull, rc::Rc};

use inkwell::{
    builder::Builder,
    values::{BasicValueEnum, FunctionValue, IntValue},
};

pub struct Scope<'ctx> {
    variables: HashMap<String, IntValue<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
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
        self.variables.insert(name.to_string(), val);
    }

    pub fn get_variable(&self, name: &str) -> Option<IntValue<'ctx>> {
        self.variables.get(name).cloned()
    }
}
