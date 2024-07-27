use std::fmt::Debug;

use llvm_sys::target::{
    LLVMCopyStringRepOfTargetData, LLVMDisposeTargetData, LLVMOpaqueTargetData,
};

use crate::util::LLVMString;

#[repr(transparent)]
pub struct TargetData {
    ptr: *mut LLVMOpaqueTargetData,
}

impl TargetData {
    pub unsafe fn from_raw(ptr: *mut LLVMOpaqueTargetData) -> Self {
        Self { ptr }
    }

    pub fn raw(&self) -> *mut LLVMOpaqueTargetData {
        self.ptr
    }

    pub fn to_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMCopyStringRepOfTargetData(self.ptr)) }
    }
}

impl Debug for TargetData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.to_string();

        for chunk in str.as_bytes().utf8_chunks() {
            write!(f, "{}", chunk.valid())?;
            write!(f, "{}", chunk.invalid().escape_ascii())?;
        }

        Ok(())
    }
}

impl Drop for TargetData {
    fn drop(&mut self) {
        unsafe { LLVMDisposeTargetData(self.ptr) }
    }
}
