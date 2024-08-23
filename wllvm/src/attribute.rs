mod re_exports;

use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMCreateEnumAttribute, LLVMCreateTypeAttribute},
    LLVMOpaqueAttributeRef,
};

pub use re_exports::AttrKind;
use re_exports::UnpackedAttrKind;

use crate::Context;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Attribute<'ctx> {
    ptr: *mut LLVMOpaqueAttributeRef,
    _phantomdata: PhantomData<&'ctx LLVMOpaqueAttributeRef>,
}

impl<'ctx> Attribute<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMOpaqueAttributeRef) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn raw(&self) -> *mut LLVMOpaqueAttributeRef {
        self.ptr
    }
}

impl Context {
    pub fn attribute<'ctx>(&'ctx self, kind: AttrKind<'ctx>) -> Attribute<'ctx> {
        unsafe {
            Attribute::from_raw(match kind.unpack() {
                UnpackedAttrKind::Enum(id) => LLVMCreateEnumAttribute(self.raw(), id, 0),
                UnpackedAttrKind::Type(id, val) => {
                    LLVMCreateTypeAttribute(self.raw(), id, val.raw())
                }
                UnpackedAttrKind::Int(id, val) => LLVMCreateEnumAttribute(self.raw(), id, val),
            })
        }
    }
}
