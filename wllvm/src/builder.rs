use std::{ffi::CStr, marker::PhantomData};

use llvm_sys::{
    core::{LLVMBuildAdd, LLVMDisposeBuilder, LLVMPositionBuilderAtEnd},
    LLVMBuilder,
};

use crate::{basic_block::BasicBlock, value::IntValue};

#[repr(transparent)]
pub struct Builder<'ctx> {
    ptr: *mut LLVMBuilder,
    _phantomdata: PhantomData<&'ctx LLVMBuilder>,
}

impl<'ctx> Builder<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMBuilder) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn build_add(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildAdd(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn position_at_end(&self, block: BasicBlock<'ctx>) {
        unsafe { LLVMPositionBuilderAtEnd(self.ptr, block.raw()) }
    }
}

impl<'ctx> Drop for Builder<'ctx> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.ptr) }
    }
}
