use core::ffi::c_char;
use std::{ffi::CStr, fmt::Debug, ops::Deref};

use llvm_sys::core::LLVMDisposeMessage;

/// An owned string obtained from LLVM. `LLVMDisposeMessage` is automatically used to free this string.
#[repr(transparent)]
pub struct LLVMString {
    ptr: *mut c_char,
}

impl LLVMString {
    pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
        Self { ptr }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let cstr = &**self;

        cstr.to_bytes()
    }

    pub fn as_bytes_mut(&mut self) -> &[u8] {
        let cstr = &**self;
        let len = cstr.count_bytes();

        let ptr = self.ptr.cast::<u8>();

        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl Drop for LLVMString {
    fn drop(&mut self) {
        unsafe { LLVMDisposeMessage(self.ptr) }
    }
}

impl Deref for LLVMString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.ptr) }
    }
}

impl AsRef<CStr> for LLVMString {
    fn as_ref(&self) -> &CStr {
        &*self
    }
}

impl Debug for LLVMString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}
