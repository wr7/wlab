use std::{ffi::CStr, fmt::Debug, marker::PhantomData};

use llvm_sys::{core::LLVMPrintTypeToString, LLVMType};

use crate::util::LLVMString;

/// An LLVM type reference
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Type<'ctx> {
    ptr: *mut LLVMType,
    _phantomdata: PhantomData<&'ctx LLVMType>,
}

impl<'ctx> Type<'ctx> {
    /// Creates a type from a raw `LLVMType` pointer.
    /// # Safety
    /// - The context of the pointed-to type must live for `'ctx`.
    pub unsafe fn from_raw(ptr: *mut LLVMType) -> Self {
        Type {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    /// Prints the type into an [`LLVMString`].
    pub fn to_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMPrintTypeToString(self.ptr)) }
    }
}

impl Debug for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cstr: &CStr = &*self.to_string();

        Debug::fmt(cstr, f)
    }
}
