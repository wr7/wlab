use std::{borrow::Borrow, collections::HashMap};

use wllvm::{type_::StructType, value::FnValue};

use crate::{
    codegen::{self, types::Type},
    error_handling::{Diagnostic, Spanned},
    parser::ast,
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
    pub function: FnValue<'ctx>,
    pub visibility: ast::Visibility,
}

pub struct FieldInfo {
    pub name: String,
    pub ty: Type,
    pub line_no: u32,
}

pub struct StructInfo<'ctx> {
    /// The LLVM representation of the type (or `None` if it is uninstantiable)
    pub llvm_type: Option<StructType<'ctx>>,
    pub fields: Vec<FieldInfo>,
    pub packed: bool,
    pub line_no: u32,
    pub file_no: usize,
}

pub struct NameStore<'ctx> {
    store: HashMap<String, NameStoreEntry<'ctx>>,
}

pub enum NameStoreEntry<'ctx> {
    Module(NameStore<'ctx>),
    Function(FunctionInfo<'ctx>),
    Struct(StructInfo<'ctx>),
}

impl<'ctx> NameStoreEntry<'ctx> {
    pub fn as_function(&self) -> Option<&FunctionInfo<'ctx>> {
        if let NameStoreEntry::Function(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_struct(&self) -> Option<&StructInfo<'ctx>> {
        if let NameStoreEntry::Struct(struct_) = self {
            Some(struct_)
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
        self.add_item(key, NameStoreEntry::Function(func))
    }

    /// Returns false if the struct already exists
    pub fn add_struct<S>(&mut self, key: &[S], struct_: StructInfo<'ctx>) -> bool
    where
        S: Borrow<str>,
    {
        self.add_item(key, NameStoreEntry::Struct(struct_))
    }

    /// Returns false if the item already exists
    pub fn add_item<S>(&mut self, key: &[S], item: NameStoreEntry<'ctx>) -> bool
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
                _ => unreachable!(),
            }
        }

        if parent.contains_key(funcname) {
            return false;
        }

        parent.insert(funcname.to_owned(), item);
        true
    }

    pub fn get_item<S>(&self, key: &[Spanned<S>]) -> Result<&NameStoreEntry<'ctx>, Diagnostic>
    where
        S: Borrow<str>,
    {
        let Some((item_name, parents)) = key.split_last() else {
            unreachable!()
        };

        let item_name: Spanned<&str> = Spanned(item_name.0.borrow(), item_name.1);

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

        let Some(item) = parent.get(*item_name) else {
            return Err(codegen::error::no_item(parent_name, item_name));
        };

        Ok(item)
    }

    pub fn get_item_from_string(&self, key: &str) -> Option<&NameStoreEntry<'ctx>> {
        let (parents, funcname) = key.rsplit_once("::").unwrap_or((&key[0..0], key));

        let mut parent = &self.store;

        for pmod in parents.split("::") {
            match parent.get(pmod) {
                Some(NameStoreEntry::Module(store)) => parent = &store.store,
                _ => return None,
            }
        }

        parent.get(funcname)
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
            .ok_or_else(|| codegen::error::undefined_item(item_name))
    }

    pub fn get_item_in_crate_mut(
        &mut self,
        crate_name: &str,
        item_name: &str,
    ) -> &mut NameStoreEntry<'ctx> {
        let Some(NameStoreEntry::Module(crate_)) = self.store.get_mut(crate_name) else {
            unreachable!()
        };

        crate_.store.get_mut(item_name).unwrap()
    }
}
