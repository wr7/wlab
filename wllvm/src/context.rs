use std::{
    ffi::{c_char, CStr},
    ptr,
};

use llvm_sys::{
    core::{
        LLVMConstStringInContext, LLVMConstStructInContext, LLVMContextCreate, LLVMContextDispose,
        LLVMCreateBuilderInContext, LLVMFunctionType, LLVMInsertBasicBlockInContext,
        LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMMoveBasicBlockAfter,
        LLVMPointerTypeInContext, LLVMStructTypeInContext,
    },
    debuginfo::LLVMDIBuilderCreateDebugLocation,
    prelude::LLVMBool,
    target::LLVMIntPtrTypeInContext,
    LLVMContext, LLVMType, LLVMValue,
};

use crate::{
    debug_info::{DILocation, DIScope},
    target::TargetData,
    type_::{FnType, IntType, PtrType, StructType},
    util,
    value::{ArrayValue, StructValue, Value},
    BasicBlock, Builder, Module, Type,
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

    pub fn insert_basic_block_after<'ctx>(
        &'ctx self,
        bb: BasicBlock<'ctx>,
        name: &CStr,
    ) -> BasicBlock<'ctx> {
        unsafe {
            let new_bb = LLVMInsertBasicBlockInContext(self.ptr, bb.raw(), name.as_ptr());
            LLVMMoveBasicBlockAfter(new_bb, bb.raw());

            BasicBlock::from_raw(new_bb)
        }
    }

    /// Creates a new DebugLocation that describes a source location.
    ///
    /// Note: If the item to which this location is attached cannot be
    /// attributed to a source line, pass 0 for the line and column.
    ///
    /// * `Line` - The line in the source file.
    /// * `Column` - The column in the source file.
    /// * `Scope` - The scope in which the location resides.
    /// * `InlinedAt` - The scope where this location was inlined, if at all. (optional).
    pub fn debug_location<'ctx>(
        &'ctx self,
        line: u32,
        column: u32,
        scope: DIScope<'ctx>,
        inlined_at: Option<DILocation<'ctx>>,
    ) -> DILocation<'ctx> {
        unsafe {
            DILocation::from_raw(LLVMDIBuilderCreateDebugLocation(
                self.ptr,
                line,
                column,
                scope.raw(),
                inlined_at.map_or(ptr::null_mut(), |l| l.raw()),
            ))
        }
    }

    pub fn const_string<'ctx>(
        &'ctx self,
        string: &(impl ?Sized + AsRef<[u8]>),
        null_terminate: bool,
    ) -> ArrayValue<'ctx> {
        let string = string.as_ref();
        let string_ptr = string.as_ptr().cast::<c_char>();

        unsafe {
            ArrayValue::from_raw(LLVMConstStringInContext(
                self.ptr,
                string_ptr,
                string.len() as u32,
                (!null_terminate) as LLVMBool,
            ))
        }
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
