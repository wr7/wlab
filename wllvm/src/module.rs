use std::{
    ffi::{c_char, CStr},
    marker::PhantomData,
    mem::MaybeUninit,
};

use llvm_sys::{
    core::{LLVMAddFunction, LLVMDisposeModule, LLVMPrintModuleToFile, LLVMPrintModuleToString},
    LLVMModule,
};

use crate::{
    type_::FnType,
    util::{LLVMErrorString, LLVMString},
    value::FnValue,
};

/// An LLVM Module
#[repr(transparent)]
pub struct Module<'ctx> {
    ptr: *mut LLVMModule,
    _phantomdata: PhantomData<&'ctx LLVMModule>,
}

impl<'ctx> Module<'ctx> {
    /// Wraps a raw LLVMModule pointer,
    /// # Safety
    /// `'ctx` cannot outlive the context of the module
    pub unsafe fn from_raw(ptr: *mut LLVMModule) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    /// Creates a function in the module
    pub fn add_function(&self, name: &CStr, function_type: FnType<'ctx>) -> FnValue<'ctx> {
        unsafe {
            FnValue::from_raw(LLVMAddFunction(
                self.ptr,
                name.as_ptr(),
                function_type.raw(),
            ))
        }
    }

    /// Converts the module to human-readable LLVM IR and then prints it to a string
    pub fn print_to_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMPrintModuleToString(self.ptr)) }
    }

    /// Converts the module to human-readable LLVM IR and then writes it to a file
    pub fn print_to_file(&self, filename: &CStr) -> Result<(), LLVMErrorString> {
        let mut err_msg: MaybeUninit<*mut c_char> = MaybeUninit::uninit();

        let result =
            unsafe { LLVMPrintModuleToFile(self.ptr, filename.as_ptr(), err_msg.as_mut_ptr()) };

        if result != 0 {
            unsafe { Err(LLVMErrorString::from_raw(err_msg.assume_init())) }
        } else {
            Ok(())
        }
    }
}

impl<'ctx> Drop for Module<'ctx> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.ptr) }
    }
}
