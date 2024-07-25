use std::ffi::CStr;

use llvm_sys::{
    core::{
        LLVMContextCreate, LLVMContextDispose, LLVMFunctionType, LLVMIntTypeInContext,
        LLVMModuleCreateWithNameInContext, LLVMStructTypeInContext,
    },
    prelude::LLVMBool,
    LLVMContext, LLVMType,
};

use crate::{
    type_::{FnType, IntType, StructType},
    Module, Type,
};

/// An LLVM Context
pub struct Context {
    ptr: *mut LLVMContext,
}

impl Context {
    pub fn new() -> Self {
        let ptr = unsafe { LLVMContextCreate() };
        Self { ptr }
    }

    pub fn create_module<'ctx>(&'ctx self, name: &CStr) -> Module<'ctx> {
        unsafe { Module::from_raw(LLVMModuleCreateWithNameInContext(name.as_ptr(), self.ptr)) }
    }

    pub fn struct_type<'ctx>(
        &'ctx self,
        elements: &[Type<'ctx>],
        packed: bool,
    ) -> StructType<'ctx> {
        let elements_ptr = elements.as_ptr().cast::<*mut LLVMType>().cast_mut();

        unsafe {
            StructType::<'ctx>::from_raw(LLVMStructTypeInContext(
                self.ptr,
                elements_ptr,
                elements.len() as u32,
                packed as LLVMBool,
            ))
        }
    }

    pub fn fn_type<'ctx>(
        &'ctx self,
        return_type: Type<'ctx>,
        param_types: &[Type<'ctx>],
        is_var_arg: bool,
    ) -> FnType<'ctx> {
        let param_types_ptr = param_types.as_ptr().cast::<*mut LLVMType>().cast_mut();

        unsafe {
            FnType::<'ctx>::from_raw(LLVMFunctionType(
                return_type.into_raw(),
                param_types_ptr,
                param_types.len() as u32,
                is_var_arg as LLVMBool,
            ))
        }
    }

    pub fn int_type<'ctx>(&'ctx self, num_bits: u32) -> IntType<'ctx> {
        unsafe { IntType::<'ctx>::from_raw(LLVMIntTypeInContext(self.ptr, num_bits)) }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.ptr) }
    }
}
