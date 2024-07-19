use std::{borrow::Borrow, collections::HashMap};

use inkwell::values::FunctionValue;

use crate::{
    codegen::{self, types::Type},
    error_handling::{Diagnostic, Spanned},
    util::HashMapExt,
};

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub return_type: Type,
}

#[derive(Clone, Debug)]
pub struct FunctionInfo<'ctx> {
    pub signature: FunctionSignature,
    pub function: FunctionValue<'ctx>,
    pub name: String,
}

pub struct NameStore<'ctx> {
    store: HashMap<String, NameStoreEntry<'ctx>>,
}

pub enum NameStoreEntry<'ctx> {
    Module(NameStore<'ctx>),
    Function(FunctionInfo<'ctx>),
}

impl<'ctx> NameStoreEntry<'ctx> {
    pub fn as_function(&self) -> Option<&FunctionInfo<'ctx>> {
        if let NameStoreEntry::Function(func) = self {
            Some(func)
        } else {
            None
        }
    }
}

impl<'ctx> NameStore<'ctx> {
    pub fn new() -> Self {
        let store = HashMap::new();
        Self { store }
    }

    /// Returns false if the function already exists
    pub fn add_function<S>(&mut self, key: &[S], func: FunctionInfo<'ctx>) -> bool
    where
        S: Borrow<str>,
    {
        let Some((funcname, parents)) = key.split_last() else {
            unreachable!()
        };

        let funcname: &str = funcname.borrow();

        let mut parent = &mut self.store;

        for p in parents {
            let p: &str = p.borrow();

            match parent.get_or_insert_with_mut(p, || NameStoreEntry::Module(NameStore::new())) {
                NameStoreEntry::Module(store) => parent = &mut store.store,
                NameStoreEntry::Function(_) => unreachable!(),
            }
        }

        if parent.contains_key(funcname) {
            return false;
        }

        parent.insert(funcname.to_owned(), NameStoreEntry::Function(func));
        true
    }

    pub fn get_item<S>(&self, key: &[Spanned<S>]) -> Result<&NameStoreEntry<'ctx>, Diagnostic>
    where
        S: Borrow<str>,
    {
        let Some((funcname, parents)) = key.split_last() else {
            unreachable!()
        };

        let funcname: Spanned<&str> = Spanned(funcname.0.borrow(), funcname.1);

        let mut parent = &self.store;
        let mut parent_name = None;

        for pmod in parents {
            let pmod: Spanned<&str> = Spanned(pmod.0.borrow(), pmod.1);

            match parent.get(*pmod) {
                Some(NameStoreEntry::Module(store)) => parent = &store.store,
                None => return Err(codegen::error::no_item(parent_name, pmod)),
                _ => return Err(codegen::error::not_module(pmod)),
            }

            parent_name = Some(*pmod);
        }

        let Some(item) = parent.get(*funcname) else {
            return Err(codegen::error::no_item(parent_name, funcname));
        };

        Ok(item)
    }

    pub fn get_item_in_crate(
        &self,
        crate_name: &str,
        item_name: Spanned<&str>,
    ) -> Result<&NameStoreEntry<'ctx>, Diagnostic> {
        let Some(NameStoreEntry::Module(crate_)) = self.store.get(crate_name) else {
            unreachable!()
        };

        crate_
            .store
            .get(*item_name)
            .ok_or_else(|| codegen::error::undefined_function(item_name))
    }
}
