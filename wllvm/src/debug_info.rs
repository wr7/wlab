use std::marker::PhantomData;

use llvm_sys::{
    debuginfo::{LLVMDIBuilderFinalize, LLVMDisposeDIBuilder},
    LLVMOpaqueDIBuilder,
};

pub use metadata::*;
mod metadata;

#[repr(transparent)]
pub struct DIBuilder<'ctx> {
    ptr: *mut LLVMOpaqueDIBuilder,
    _phantomdata: PhantomData<&'ctx LLVMOpaqueDIBuilder>,
}

impl<'ctx> DIBuilder<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMOpaqueDIBuilder) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn finalize(&self) {
        unsafe { LLVMDIBuilderFinalize(self.ptr) }
    }
}

impl<'ctx> Drop for DIBuilder<'ctx> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeDIBuilder(self.ptr) }
    }
}
