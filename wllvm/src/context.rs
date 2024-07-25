use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose, LLVMStructTypeInContext},
    prelude::LLVMBool,
    LLVMContext, LLVMType,
};

use crate::Type;

/// An LLVM Context
pub struct Context {
    ptr: *mut LLVMContext,
}

impl Context {
    pub fn new() -> Self {
        let ptr = unsafe { LLVMContextCreate() };
        Self { ptr }
    }

    pub fn struct_type<'ctx>(&'ctx self, elements: &[Type<'ctx>], packed: bool) -> Type<'ctx> {
        let elements_ptr = elements.as_ptr().cast::<*mut LLVMType>().cast_mut();

        let type_ = unsafe {
            Type::<'ctx>::from_raw(LLVMStructTypeInContext(
                self.ptr,
                elements_ptr,
                elements.len() as u32,
                packed as LLVMBool,
            ))
        };

        type_
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.ptr) }
    }
}
