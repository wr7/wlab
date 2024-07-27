use std::ffi::CStr;

use llvm_sys::{
    core::{
        LLVMConstStructInContext, LLVMContextCreate, LLVMContextDispose,
        LLVMCreateBuilderInContext, LLVMFunctionType, LLVMIntTypeInContext,
        LLVMModuleCreateWithNameInContext, LLVMPointerTypeInContext, LLVMStructTypeInContext,
    },
    prelude::LLVMBool,
    target::LLVMIntPtrTypeInContext,
    LLVMContext, LLVMType, LLVMValue,
};

use crate::{
    target::TargetData,
    type_::{FnType, IntType, PtrType, StructType},
    util,
    value::{StructValue, Value},
    Builder, Module, Type,
};

/// An LLVM Context
pub struct Context {
    ptr: *mut LLVMContext,
}

impl Context {
    pub unsafe fn from_raw(ptr: *mut LLVMContext) -> Self {
        Self { ptr }
    }

    pub unsafe fn from_raw_ref<'a>(raw: &'a *mut LLVMContext) -> &'a Self {
        util::transmute_ref::<*mut LLVMContext, Self>(raw)
    }

    pub fn raw(&self) -> *mut LLVMContext {
        self.ptr
    }

    pub fn new() -> Self {
        unsafe { Self::from_raw(LLVMContextCreate()) }
    }

    pub fn create_module<'ctx>(&'ctx self, name: &CStr) -> Module<'ctx> {
        unsafe { Module::from_raw(LLVMModuleCreateWithNameInContext(name.as_ptr(), self.ptr)) }
    }

    pub fn create_builder<'ctx>(&'ctx self) -> Builder<'ctx> {
        unsafe { Builder::from_raw(LLVMCreateBuilderInContext(self.ptr)) }
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

    pub fn const_struct<'ctx>(
        &'ctx self,
        elements: &[Value<'ctx>],
        packed: bool,
    ) -> StructValue<'ctx> {
        let elements_ptr = elements.as_ptr().cast::<*mut LLVMValue>().cast_mut();

        unsafe {
            StructValue::from_raw(LLVMConstStructInContext(
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
                return_type.raw(),
                param_types_ptr,
                param_types.len() as u32,
                is_var_arg as LLVMBool,
            ))
        }
    }

    pub fn int_type<'ctx>(&'ctx self, num_bits: u32) -> IntType<'ctx> {
        unsafe { IntType::<'ctx>::from_raw(LLVMIntTypeInContext(self.ptr, num_bits)) }
    }

    pub fn ptr_sized_int_type<'ctx>(&'ctx self, target_data: &TargetData) -> IntType<'ctx> {
        unsafe { IntType::from_raw(LLVMIntPtrTypeInContext(self.ptr, target_data.raw())) }
    }

    pub fn ptr_type<'ctx>(&'ctx self) -> PtrType<'ctx> {
        unsafe { PtrType::from_raw(LLVMPointerTypeInContext(self.ptr, 0)) }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.ptr) }
    }
}
