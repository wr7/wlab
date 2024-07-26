use std::marker::PhantomData;

use llvm_sys::LLVMBasicBlock;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct BasicBlock<'ctx> {
    ptr: *mut LLVMBasicBlock,
    _phantomdata: PhantomData<&'ctx LLVMBasicBlock>,
}

impl<'ctx> BasicBlock<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMBasicBlock) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn raw(&self) -> *mut LLVMBasicBlock {
        self.ptr
    }
}
